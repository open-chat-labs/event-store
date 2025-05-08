use candid::{CandidType, Deserialize, Principal};
use event_store_types::Milliseconds;
use serde::Serialize;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct InitArgs {
    pub push_events_whitelist: Vec<Principal>,
    pub read_events_whitelist: Vec<Principal>,
    pub time_granularity: Option<Milliseconds>,
}
