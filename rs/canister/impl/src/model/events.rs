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
        let start_index = start as usize;
        if start_index < self.events.len() {
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

    pub fn stats(&self) -> EventsStats {
        EventsStats {
            latest_event_index: self.latest_event_index,
        }
    }
}

pub struct EventsStats {
    pub latest_event_index: Option<u64>,
}
