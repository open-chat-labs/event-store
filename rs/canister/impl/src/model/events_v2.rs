use crate::memory::{get_events_v2_data_memory, get_events_v2_index_memory, Memory};
use crate::model::string_to_num_map::StringToNumMap;
use candid::Deserialize;
use event_store_canister::{IdempotentEvent, IndexedEvent, TimestampMillis};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{StableLog, Storable};
use serde::Serialize;
use std::borrow::Cow;

#[derive(Serialize, Deserialize)]
pub struct EventsV2 {
    #[serde(skip, default = "init_events")]
    events: StableLog<StorableEvent, Memory, Memory>,
    #[serde(default)]
    string_to_num_map: StringToNumMap,
}

impl EventsV2 {
    pub fn get(&self, start: u64, length: u64) -> Vec<IndexedEvent> {
        self.events
            .iter()
            .skip(start as usize)
            .take(length as usize)
            .map(|e| self.hydrate(e))
            .collect()
    }

    pub fn push(&mut self, event: IdempotentEvent) {
        let storable = self.convert_to_storable(event, self.events.len());

        self.events.append(&storable).unwrap();
    }

    pub fn stats(&self) -> EventsStats {
        EventsStats {
            latest_event_index: self.events.len().checked_sub(1),
        }
    }

    pub fn len(&self) -> u64 {
        self.events.len()
    }

    fn convert_to_storable(&mut self, event: IdempotentEvent, index: u64) -> StorableEvent {
        StorableEvent {
            index,
            name: self.string_to_num_map.convert_to_num(event.name),
            timestamp: event.timestamp,
            user: event.user.map(|u| self.string_to_num_map.convert_to_num(u)),
            source: event
                .source
                .map(|s| self.string_to_num_map.convert_to_num(s)),
            payload: event.payload,
        }
    }

    fn hydrate(&self, event: StorableEvent) -> IndexedEvent {
        IndexedEvent {
            index: event.index,
            name: self
                .string_to_num_map
                .convert_to_string(event.name)
                .unwrap_or("unknown".to_string()),
            timestamp: event.timestamp,
            user: event
                .user
                .and_then(|u| self.string_to_num_map.convert_to_string(u)),
            source: event
                .source
                .and_then(|s| self.string_to_num_map.convert_to_string(s)),
            payload: event.payload,
        }
    }
}

impl Default for EventsV2 {
    fn default() -> Self {
        EventsV2 {
            events: init_events(),
            string_to_num_map: StringToNumMap::default(),
        }
    }
}

fn init_events() -> StableLog<StorableEvent, Memory, Memory> {
    StableLog::init(get_events_v2_index_memory(), get_events_v2_data_memory()).unwrap()
}

pub struct EventsStats {
    pub latest_event_index: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct StorableEvent {
    #[serde(rename = "i")]
    index: u64,
    #[serde(rename = "n")]
    name: u32,
    #[serde(rename = "t")]
    timestamp: TimestampMillis,
    #[serde(rename = "u", default, skip_serializing_if = "Option::is_none")]
    user: Option<u32>,
    #[serde(rename = "s", default, skip_serializing_if = "Option::is_none")]
    source: Option<u32>,
    #[serde(
        rename = "p",
        default,
        skip_serializing_if = "is_empty_slice",
        with = "serde_bytes"
    )]
    payload: Vec<u8>,
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
