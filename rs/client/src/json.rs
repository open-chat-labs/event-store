use crate::Runtime;
use event_sink_canister::TimestampMillis;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::Deref;

type Client<R> = EventSinkClient<R>;
type ClientBuilder<R> = EventSinkClientBuilder<R>;

pub struct EventSinkClient<R> {
    inner: super::Client<R>,
}

pub struct EventSinkClientBuilder<R> {
    inner: super::ClientBuilder<R>,
}

impl<R: Runtime + Send + 'static> ClientBuilder<R> {
    pub(crate) fn new(inner: super::ClientBuilder<R>) -> Self {
        ClientBuilder { inner }
    }

    pub fn build(self) -> Client<R> {
        let inner = self.inner.build();

        Client { inner }
    }
}

pub struct Event<P> {
    pub name: String,
    pub timestamp: TimestampMillis,
    pub user: Option<String>,
    pub source: Option<String>,
    pub payload: P,
}

impl<R: Runtime + Send + 'static> Client<R> {
    pub fn push<P: Serialize>(&mut self, event: Event<P>) {
        self.inner.push(event.into())
    }

    pub fn push_many<P: Serialize>(&mut self, events: impl Iterator<Item = Event<P>>) {
        self.inner.push_many(events.into_iter().map(|e| e.into()))
    }
}

impl<R> Deref for Client<R> {
    type Target = super::Client<R>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<P: Serialize> From<Event<P>> for super::Event {
    fn from(value: Event<P>) -> Self {
        super::Event {
            name: value.name,
            timestamp: value.timestamp,
            user: value.user,
            source: value.source,
            payload: serde_json::to_vec(&value.payload).unwrap(),
        }
    }
}

impl<R: Serialize> Serialize for Client<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de, R: Deserialize<'de> + Runtime + Send + 'static> Deserialize<'de> for Client<R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = super::Client::deserialize(deserializer)?;

        Ok(Client { inner })
    }
}
