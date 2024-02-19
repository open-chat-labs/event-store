use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct WhitelistedPrincipals {
    pub read: Vec<Principal>,
    pub push: Vec<Principal>,
    pub remove: Vec<Principal>,
}
