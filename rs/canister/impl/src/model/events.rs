use crate::memory::{get_events_data_memory, get_events_index_memory, Memory};
use crate::model::string_to_num_map::StringToNumMap;
use candid::Deserialize;
use event_store_canister::{Anonymizable, IdempotentEvent, IndexedEvent, TimestampMillis};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{StableLog, Storable};
use serde::Serialize;
use sha2::Digest;
use std::borrow::Cow;
use std::fmt::Write;

pub struct Events {
    events: StableLog<StorableEvent, Memory, Memory>,
    string_to_num_map: StringToNumMap,
}

impl Events {
    pub fn get(&self, start: u64, length: u64) -> Vec<IndexedEvent> {
        self.events
            .iter()
            .skip(start as usize)
            .take(length as usize)
            .map(|e| self.hydrate(e))
            .collect()
    }

    pub fn push(&mut self, event: IdempotentEvent, salt: [u8; 32]) {
        let storable = self.convert_to_storable(event, self.events.len(), salt);

        self.events.append(&storable).unwrap();
    }

    pub fn stats(&self) -> EventsStats {
        EventsStats {
            latest_event_index: self.events.len().checked_sub(1),
        }
    }

    fn convert_to_storable(
        &mut self,
        event: IdempotentEvent,
        index: u64,
        salt: [u8; 32],
    ) -> StorableEvent {
        StorableEvent {
            index,
            name: self.string_to_num_map.convert_to_num(event.name),
            timestamp: event.timestamp,
            user: event
                .user
                .map(|u| to_maybe_anonymized_string(u, salt))
                .map(|u| self.string_to_num_map.convert_to_num(u)),
            source: event
                .source
                .map(|s| to_maybe_anonymized_string(s, salt))
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

impl Default for Events {
    fn default() -> Self {
        Events {
            events: init_events(),
            string_to_num_map: StringToNumMap::default(),
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

fn to_maybe_anonymized_string(value: Anonymizable, salt: [u8; 32]) -> String {
    match value {
        Anonymizable::Public(s) => s,
        Anonymizable::Anonymize(s) => anonymize(&s, salt),
    }
}

fn anonymize(value: &str, salt: [u8; 32]) -> String {
    // Generates a 32 character string from the input value + the salt
    let mut hasher = sha2::Sha256::new();
    hasher.update(value.as_bytes());
    hasher.update(salt);
    let hash: [u8; 32] = hasher.finalize().into();

    let mut string = String::with_capacity(32);
    for byte in &hash[16..] {
        write!(string, "{byte:02x}").unwrap();
    }
    string
}
