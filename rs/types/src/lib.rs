use candid::CandidType;
use serde::{Deserialize, Serialize};

pub type Milliseconds = u64;
pub type TimestampMillis = u64;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    name: String,
    timestamp: TimestampMillis,
    user: Option<Anonymizable>,
    source: Option<Anonymizable>,
    payload: Vec<u8>,
}

pub struct EventBuilder {
    name: String,
    timestamp: TimestampMillis,
    user: Option<Anonymizable>,
    source: Option<Anonymizable>,
    payload: Vec<u8>,
}

impl EventBuilder {
    pub fn new(name: impl Into<String>, timestamp: TimestampMillis) -> Self {
        Self {
            name: name.into(),
            timestamp,
            user: None,
            source: None,
            payload: Vec::new(),
        }
    }

    pub fn with_user(mut self, user: impl Into<String>, anonymize: bool) -> Self {
        self.user = Some(Anonymizable::new(user.into(), anonymize));
        self
    }

    pub fn with_maybe_user(mut self, user: Option<impl Into<String>>, anonymize: bool) -> Self {
        self.user = user.map(|u| Anonymizable::new(u.into(), anonymize));
        self
    }

    pub fn with_source(mut self, source: impl Into<String>, anonymize: bool) -> Self {
        self.source = Some(Anonymizable::new(source.into(), anonymize));
        self
    }

    pub fn with_maybe_source(mut self, source: Option<impl Into<String>>, anonymize: bool) -> Self {
        self.source = source.map(|u| Anonymizable::new(u.into(), anonymize));
        self
    }

    pub fn with_payload(mut self, payload: Vec<u8>) -> Self {
        self.payload = payload;
        self
    }

    #[cfg(feature = "json")]
    pub fn with_json_payload<P: Serialize>(self, payload: &P) -> Self {
        self.with_payload(serde_json::to_vec(payload).unwrap())
    }

    pub fn build(self) -> Event {
        Event {
            name: self.name,
            timestamp: self.timestamp,
            user: self.user,
            source: self.source,
            payload: self.payload,
        }
    }
}

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

    pub fn as_str(&self) -> &str {
        match self {
            Anonymizable::Public(s) => s,
            Anonymizable::Anonymize(s) => s,
        }
    }

    pub fn is_public(&self) -> bool {
        matches!(self, Anonymizable::Public(_))
    }
}

impl Event {
    pub fn to_idempotent(self, idempotency_key: u128) -> IdempotentEvent {
        IdempotentEvent {
            idempotency_key,
            name: self.name,
            timestamp: self.timestamp,
            user: self.user,
            source: self.source,
            payload: self.payload,
        }
    }
}

impl From<IdempotentEvent> for Event {
    fn from(value: IdempotentEvent) -> Self {
        Event {
            name: value.name,
            timestamp: value.timestamp,
            user: value.user,
            source: value.source,
            payload: value.payload,
        }
    }
}
