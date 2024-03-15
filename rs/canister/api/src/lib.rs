use candid::{CandidType, Deserialize};
use serde::Serialize;

mod lifecycle;
mod queries;
mod updates;

pub use lifecycle::*;
pub use queries::*;
pub use updates::*;

pub type TimestampMillis = u64;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct IdempotentEvent {
    pub idempotency_key: u128,
    pub name: String,
    pub timestamp: TimestampMillis,
    pub user: Option<Anonymizable>,
    pub source: Option<Anonymizable>,
    #[serde(with = "serde_bytes")]
    pub payload: Vec<u8>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct IndexedEvent {
    pub index: u64,
    pub name: String,
    pub timestamp: TimestampMillis,
    pub user: Option<String>,
    pub source: Option<String>,
    #[serde(with = "serde_bytes")]
    pub payload: Vec<u8>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum Anonymizable {
    Public(String),
    Anonymize(String),
}

impl Anonymizable {
    pub fn new(value: String, anonymize: bool) -> Anonymizable {
        if anonymize {
            Anonymizable::Anonymize(value)
        } else {
            Anonymizable::Public(value)
        }
    }
}
