use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct RemoveEventsArgs {
    pub up_to_inclusive: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct RemoveEventsResponse {
    pub latest_event_index: Option<u64>,
    pub earliest_event_index_stored: Option<u64>,
}
