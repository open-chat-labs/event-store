use candid::Deserialize;
use event_sink_canister::{IdempotentEvent, IndexedEvent, TimestampMillis};
use serde::Serialize;
use std::collections::btree_map::Entry::Vacant;
use std::collections::{BTreeMap, VecDeque};

const RECENTLY_ADDED_PRUNE_INTERVAL_MS: u64 = 30 * 60 * 1000; // 30 minutes
const RECENTLY_ADDED_WINDOW_MS: u64 = 60 * 60 * 1000; // 1 hour

#[derive(Serialize, Deserialize, Default)]
pub struct Events {
    events: VecDeque<IndexedEvent>,
    latest_event_index: Option<u64>,
    recently_added: BTreeMap<u128, TimestampMillis>,
    recently_added_last_pruned: TimestampMillis,
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

    pub fn push(&mut self, event: IdempotentEvent, now: TimestampMillis) {
        match self.recently_added.entry(event.idempotency_key) {
            Vacant(e) => e.insert(now),
            _ => return,
        };

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

        if now.saturating_sub(self.recently_added_last_pruned) > RECENTLY_ADDED_PRUNE_INTERVAL_MS {
            self.prune_recently_added(now);
        }
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

    fn prune_recently_added(&mut self, now: TimestampMillis) {
        let cutoff = now.saturating_sub(RECENTLY_ADDED_WINDOW_MS);
        self.recently_added.retain(|_, ts| *ts > cutoff);
    }
}

pub struct EventsStats {
    pub earliest_event_index_stored: Option<u64>,
    pub latest_event_index: Option<u64>,
}
