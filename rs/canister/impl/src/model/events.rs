use crate::memory::{get_events_data_memory, get_events_index_memory, Memory};
use candid::Deserialize;
use event_sink_canister::{IdempotentEvent, IndexedEvent, TimestampMillis};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{StableLog, Storable};
use serde::Serialize;
use std::borrow::Cow;

#[derive(Serialize, Deserialize)]
pub struct Events {
    #[serde(skip, default = "init_events")]
    events: StableLog<StorableEvent, Memory, Memory>,
}

impl Events {
    pub fn get(&self, start: u64, length: u64) -> Vec<IndexedEvent> {
        self.events
            .iter()
            .skip(start as usize)
            .take(length as usize)
            .map(|e| e.into())
            .collect()
    }

    pub fn push(&mut self, event: IdempotentEvent) {
        self.events
            .append(&StorableEvent::new(event.clone(), self.events.len()))
            .unwrap();
    }

    pub fn stats(&self) -> EventsStats {
        EventsStats {
            latest_event_index: self.events.len().checked_sub(1),
        }
    }
}

impl Default for Events {
    fn default() -> Self {
        Events {
            events: init_events(),
        }
    }
}

fn init_events() -> StableLog<StorableEvent, Memory, Memory> {
    StableLog::init(get_events_index_memory(), get_events_data_memory()).unwrap()
}

pub struct EventsStats {
    pub latest_event_index: Option<u64>,
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
