use crate::memory::{get_events_data_memory, get_events_index_memory, Memory};
use candid::Deserialize;
use event_sink_canister::{IdempotentEvent, IndexedEvent, TimestampMillis};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{StableLog, Storable};
use serde::Serialize;
use std::borrow::Cow;
use std::collections::VecDeque;

#[derive(Serialize, Deserialize)]
pub struct Events {
    events: VecDeque<IndexedEvent>,
    #[serde(skip, default = "init_events")]
    events_v2: StableLog<StorableEvent, Memory, Memory>,
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

    pub fn migrate(&mut self, count: u64) {
        for event in self.get(self.events_v2.len(), count) {
            self.events_v2
                .append(&StorableEvent {
                    index: event.index,
                    name: event.name,
                    timestamp: event.timestamp,
                    user: event.user,
                    source: event.source,
                    payload: event.payload,
                })
                .unwrap();
        }
    }

    pub fn push(&mut self, event: IdempotentEvent) {
        let index = self.latest_event_index.map_or(0, |i| i + 1);

        if self.events_v2.len() == index {
            self.events_v2
                .append(&StorableEvent::new(event.clone(), index))
                .unwrap();
        }

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
            latest_event_index_in_stable_memory: self.events_v2.iter().last().map(|e| e.index),
        }
    }
}

impl Default for Events {
    fn default() -> Self {
        Events {
            events: VecDeque::new(),
            events_v2: init_events(),
            latest_event_index: None,
        }
    }
}

fn init_events() -> StableLog<StorableEvent, Memory, Memory> {
    StableLog::init(get_events_index_memory(), get_events_data_memory()).unwrap()
}

pub struct EventsStats {
    pub latest_event_index: Option<u64>,
    pub latest_event_index_in_stable_memory: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct StorableEvent {
    #[serde(rename = "i")]
    index: u64,
    #[serde(rename = "n")]
    name: String,
    #[serde(rename = "t")]
    timestamp: TimestampMillis,
    #[serde(rename = "u", default, skip_serializing_if = "Option::is_none")]
    user: Option<String>,
    #[serde(rename = "s", default, skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(
        rename = "p",
        default,
        skip_serializing_if = "is_empty_slice",
        with = "serde_bytes"
    )]
    payload: Vec<u8>,
}

impl StorableEvent {
    fn new(event: IdempotentEvent, index: u64) -> StorableEvent {
        StorableEvent {
            index,
            name: event.name,
            timestamp: event.timestamp,
            user: event.user,
            source: event.source,
            payload: event.payload,
        }
    }
}

impl From<StorableEvent> for IndexedEvent {
    fn from(value: StorableEvent) -> Self {
        IndexedEvent {
            index: value.index,
            name: value.name,
            timestamp: value.timestamp,
            user: value.user,
            source: value.source,
            payload: value.payload,
        }
    }
}

impl Storable for StorableEvent {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(rmp_serde::to_vec_named(&self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        rmp_serde::from_slice(bytes.as_ref()).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

fn is_empty_slice<T>(vec: &[T]) -> bool {
    vec.is_empty()
}
