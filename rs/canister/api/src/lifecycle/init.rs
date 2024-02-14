use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct InitArgs {
    pub push_events_whitelist: Vec<Principal>,
    pub read_events_whitelist: Vec<Principal>,
    pub remove_events_whitelist: Vec<Principal>,
}