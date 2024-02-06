use crate::IndexedEvent;
use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct EventsArgs {
    pub start: u64,
    pub length: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct EventsResponse {
    pub events: Vec<IndexedEvent>,
    pub latest_event_index: u64,
    pub events_stored: u64,
}
