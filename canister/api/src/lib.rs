use candid::{CandidType, Deserialize};
use serde::Serialize;
use serde_bytes::ByteBuf;

mod queries;
mod updates;

pub use queries::*;
pub use updates::*;

pub type TimestampMillis = u64;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    pub name: String,
    pub timestamp: TimestampMillis,
    pub payload: ByteBuf,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct IndexedEvent {
    pub index: u64,
    pub name: String,
    pub timestamp: TimestampMillis,
    pub payload: ByteBuf,
}
