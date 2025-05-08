use candid::{CandidType, Deserialize};
use event_store_types::IdempotentEvent;
use serde::Serialize;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct PushEventsArgs {
    pub events: Vec<IdempotentEvent>,
}
