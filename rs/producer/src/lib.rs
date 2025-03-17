use event_store_canister::{Anonymizable, IdempotentEvent, TimestampMillis};
use ic_principal::Principal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use std::{mem, thread};

#[cfg(test)]
mod tests;

const DEFAULT_FLUSH_DELAY: Duration = Duration::from_secs(300);
const DEFAULT_MAX_BATCH_SIZE: u32 = 1000;

pub type FlushOutcome = u8;

pub const FLUSH_OUTCOME_SUCCESS: u8 = 0;
pub const FLUSH_OUTCOME_FAILED_SHOULD_RETRY: u8 = 1;
pub const FLUSH_OUTCOME_FAILED_SHOULDNT_RETRY: u8 = 2;

pub struct EventStoreClient<R> {
    inner: Arc<Mutex<ClientInner<R>>>,
}

type Client<R> = EventStoreClient<R>;
type ClientBuilder<R> = EventStoreClientBuilder<R>;

#[derive(Serialize, Deserialize)]
struct ClientInner<R> {
    event_store_canister_id: Principal,
    runtime: R,
    flush_delay: Duration,
    max_batch_size: usize,
    events: Vec<IdempotentEvent>,
    #[serde(skip)]
    next_flush_scheduled: Option<TimestampMillis>,
    flush_in_progress: bool,
    total_events_flushed: u64,
}

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

pub trait Runtime {
    fn schedule_flush<F: FnOnce() + Send + 'static>(&mut self, delay: Duration, callback: F);
    fn flush<F: FnOnce(FlushOutcome) + Send + 'static>(
        &mut self,
        event_sync_canister_id: Principal,
        events: Vec<IdempotentEvent>,
        on_complete: F,
    );
    fn rng(&mut self) -> u128;
    fn now(&self) -> TimestampMillis;
}

impl<R> Client<R> {
    pub fn take_events(&mut self) -> Vec<IdempotentEvent> {
        mem::take(&mut self.inner.try_lock().unwrap().events)
    }

    pub fn info(&self) -> EventStoreClientInfo {
        let guard = self.inner.try_lock().unwrap();

        EventStoreClientInfo {
            event_store_canister_id: guard.event_store_canister_id,
            flush_delay: guard.flush_delay,
            max_batch_size: guard.max_batch_size as u32,
            events_pending: guard.events.len() as u32,
            flush_in_progress: guard.flush_in_progress,
            next_flush_scheduled: guard.next_flush_scheduled,
            total_events_flushed: guard.total_events_flushed,
        }
    }
}

impl Client<NullRuntime> {
    pub fn null() -> Client<NullRuntime> {
        ClientBuilder::new(Principal::anonymous(), NullRuntime).build()
    }
}

pub struct EventStoreClientBuilder<R> {
    event_store_canister_id: Principal,
    runtime: R,
    flush_delay: Option<Duration>,
    max_batch_size: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EventStoreClientInfo {
    pub event_store_canister_id: Principal,
    pub flush_delay: Duration,
    pub max_batch_size: u32,
    pub events_pending: u32,
    pub flush_in_progress: bool,
    pub next_flush_scheduled: Option<TimestampMillis>,
    pub total_events_flushed: u64,
}

impl<R: Runtime + Send + 'static> ClientBuilder<R> {
    pub fn new(event_store_canister_id: Principal, runtime: R) -> ClientBuilder<R> {
        ClientBuilder {
            event_store_canister_id,
            runtime,
            flush_delay: None,
            max_batch_size: None,
        }
    }

    pub fn with_flush_delay(mut self, duration: Duration) -> Self {
        self.flush_delay = Some(duration);
        self
    }

    pub fn with_max_batch_size(mut self, max_batch_size: u32) -> Self {
        self.max_batch_size = Some(max_batch_size);
        self
    }

    pub fn build(self) -> Client<R> {
        let flush_delay = self.flush_delay.unwrap_or(DEFAULT_FLUSH_DELAY);
        let max_batch_size = self.max_batch_size.unwrap_or(DEFAULT_MAX_BATCH_SIZE) as usize;

        Client {
            inner: Arc::new(Mutex::new(ClientInner::new(
                self.event_store_canister_id,
                self.runtime,
                flush_delay,
                max_batch_size,
            ))),
        }
    }
}

impl<R: Runtime + Send + 'static> Client<R> {
    pub fn push(&mut self, event: Event) {
        let mut guard = self.inner.try_lock().unwrap();
        let idempotency_key = guard.runtime.rng();
        guard.events.push(IdempotentEvent {
            idempotency_key,
            name: event.name,
            timestamp: event.timestamp,
            user: event.user,
            source: event.source,
            payload: event.payload,
        });
        self.process_events(guard, true);
    }

    pub fn push_many(&mut self, events: impl Iterator<Item = Event>, can_flush_immediately: bool) {
        let mut guard = self.inner.try_lock().unwrap();
        for event in events {
            let idempotency_key = guard.runtime.rng();
            guard.events.push(IdempotentEvent {
                idempotency_key,
                name: event.name,
                timestamp: event.timestamp,
                user: event.user,
                source: event.source,
                payload: event.payload,
            });
        }
        self.process_events(guard, can_flush_immediately);
    }

    fn flush_batch(&self) {
        let guard = self.inner.try_lock().unwrap();
        self.flush_batch_within_lock(guard);
    }

    fn flush_batch_within_lock(&self, mut guard: MutexGuard<ClientInner<R>>) {
        guard.next_flush_scheduled = None;

        if !guard.events.is_empty() {
            guard.flush_in_progress = true;

            let max_batch_size = guard.max_batch_size;

            let events = if guard.events.len() <= max_batch_size {
                mem::take(&mut guard.events)
            } else {
                guard.events.drain(..max_batch_size).collect()
            };

            let mut clone = self.clone();
            let event_store_canister_id = guard.event_store_canister_id;
            guard
                .runtime
                .flush(event_store_canister_id, events.clone(), move |outcome| {
                    clone.on_flush_complete(outcome, events)
                });
        }
    }

    fn process_events(&self, guard: MutexGuard<ClientInner<R>>, can_flush_immediately: bool) {
        if guard.flush_in_progress {
            return;
        }
        let max_batch_size_reached = guard.events.len() >= guard.max_batch_size;
        if max_batch_size_reached {
            if can_flush_immediately {
                self.flush_batch_within_lock(guard);
            } else {
                self.schedule_flush(guard, Duration::ZERO)
            }
        } else if guard.next_flush_scheduled.is_none() {
            let delay = guard.flush_delay;
            self.schedule_flush(guard, delay)
        }
    }

    fn schedule_flush(&self, mut guard: MutexGuard<ClientInner<R>>, delay: Duration) {
        let clone = self.clone();
        let now = guard.runtime.now();
        guard
            .runtime
            .schedule_flush(delay, move || clone.flush_batch());
        guard.next_flush_scheduled = Some(now + delay.as_millis() as u64);
    }

    fn on_flush_complete(&mut self, outcome: FlushOutcome, events: Vec<IdempotentEvent>) {
        if let Ok(guard) = self.inner.try_lock() {
            self.on_flush_within_lock(guard, outcome, events);
        } else {
            let clone = self.clone();
            thread::spawn(move || {
                let guard = clone.inner.lock().unwrap();
                clone.on_flush_within_lock(guard, outcome, events);
            });
        }
    }

    fn on_flush_within_lock(
        &self,
        mut guard: MutexGuard<ClientInner<R>>,
        outcome: FlushOutcome,
        events: Vec<IdempotentEvent>,
    ) {
        guard.flush_in_progress = false;

        match outcome {
            FLUSH_OUTCOME_SUCCESS => {
                guard.total_events_flushed = guard
                    .total_events_flushed
                    .saturating_add(events.len() as u64);
            }
            FLUSH_OUTCOME_FAILED_SHOULD_RETRY => {
                guard.events.extend(events);
            }
            _ => {}
        }

        if !guard.events.is_empty() {
            self.process_events(guard, false);
        }
    }
}

impl<R> Clone for Client<R> {
    fn clone(&self) -> Self {
        Client {
            inner: self.inner.clone(),
        }
    }
}

impl<R> ClientInner<R> {
    pub fn new(
        event_store_canister_id: Principal,
        runtime: R,
        flush_delay: Duration,
        max_batch_size: usize,
    ) -> ClientInner<R> {
        ClientInner {
            event_store_canister_id,
            runtime,
            flush_delay,
            max_batch_size,
            events: Vec::new(),
            next_flush_scheduled: None,
            flush_in_progress: false,
            total_events_flushed: 0,
        }
    }
}

impl<R: Serialize> Serialize for Client<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let inner = self.inner.try_lock().unwrap();
        inner.serialize(serializer)
    }
}

impl<'de, R: Deserialize<'de> + Runtime + Send + 'static> Deserialize<'de> for Client<R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = ClientInner::deserialize(deserializer)?;
        let any_events = !inner.events.is_empty();
        let client = Client {
            inner: Arc::new(Mutex::new(inner)),
        };

        if any_events {
            let guard = client.inner.try_lock().unwrap();
            client.process_events(guard, false);
        }

        Ok(client)
    }
}

pub struct NullRuntime;

impl Runtime for NullRuntime {
    fn schedule_flush<F: FnOnce() + Send + 'static>(&mut self, _delay: Duration, _callback: F) {}

    fn flush<F: FnOnce(FlushOutcome) + Send + 'static>(
        &mut self,
        _event_sync_canister_id: Principal,
        _events: Vec<IdempotentEvent>,
        on_complete: F,
    ) {
        on_complete(FLUSH_OUTCOME_SUCCESS)
    }

    fn rng(&mut self) -> u128 {
        0
    }

    fn now(&self) -> TimestampMillis {
        0
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
