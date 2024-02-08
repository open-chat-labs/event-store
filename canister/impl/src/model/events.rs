use candid::Deserialize;
use event_sink_canister::{IdempotentEvent, IndexedEvent};
use serde::Serialize;
use std::collections::VecDeque;

#[derive(Serialize, Deserialize, Default)]
pub struct Events {
    events: VecDeque<IndexedEvent>,
    latest_event_index: Option<u64>,
}

impl Events {
    pub fn get(&self, start: u64, length: u64) -> Vec<IndexedEvent> {
        if let Some(start_index) = self
            .earliest_event_index_stored()
            .and_then(|i| start.checked_sub(i))
            .map(|i| i as usize)
            .filter(|i| *i < self.events.len())
        {
            self.events
                .range(start_index..)
                .take(length as usize)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn push(&mut self, event: IdempotentEvent) {
        let index = self.latest_event_index.map_or(0, |i| i + 1);
        self.events.push_back(IndexedEvent {
            index,
            name: event.name,
            timestamp: event.timestamp,
            user: event.user,
            source: event.source,
            payload: event.payload,
        });
        self.latest_event_index = Some(index);
    }

    pub fn remove(&mut self, up_to_inclusive: u64) {
        while self
            .events
            .front()
            .filter(|e| e.index <= up_to_inclusive)
            .is_some()
        {
            self.events.pop_front();
        }
    }

    pub fn stats(&self) -> EventsStats {
        EventsStats {
            earliest_event_index_stored: self.earliest_event_index_stored(),
            latest_event_index: self.latest_event_index,
        }
    }

    fn earliest_event_index_stored(&self) -> Option<u64> {
        self.events.front().map(|e| e.index)
    }
}

pub struct EventsStats {
    pub earliest_event_index_stored: Option<u64>,
    pub latest_event_index: Option<u64>,
}
