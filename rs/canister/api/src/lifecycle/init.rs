use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct InitArgs {
    pub push_events_whitelist: Vec<Principal>,
    pub read_events_whitelist: Vec<Principal>,
    pub anonymization_config: Option<AnonymizationInitConfig>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Default)]
pub struct AnonymizationInitConfig {
    pub users: Option<bool>,
    pub sources: Option<bool>,
    pub exclusions: Option<Vec<String>>,
}
