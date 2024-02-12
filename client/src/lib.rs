use event_sink_canister::{IdempotentEvent, TimestampMillis};
use ic_principal::Principal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::mem;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

#[cfg(test)]
mod tests;

const DEFAULT_FLUSH_DELAY: Duration = Duration::from_secs(300);
const DEFAULT_MAX_BATCH_SIZE: u32 = 1000;

pub type FlushOutcome = u8;

pub const FLUSH_OUTCOME_SUCCESS: u8 = 0;
pub const FLUSH_OUTCOME_FAILED_SHOULD_RETRY: u8 = 1;
pub const FLUSH_OUTCOME_FAILED_SHOULDNT_RETRY: u8 = 2;

pub struct EventSinkClient<R> {
    inner: Arc<Mutex<ClientInner<R>>>,
}

type Client<R> = EventSinkClient<R>;
type ClientBuilder<R> = EventSinkClientBuilder<R>;

#[derive(Serialize, Deserialize)]
struct ClientInner<R> {
    event_sink_canister_id: Principal,
    runtime: R,
    flush_delay: Duration,
    max_batch_size: usize,
    events: Vec<IdempotentEvent>,
    next_flush_scheduled: Option<TimestampMillis>,
    #[serde(default)]
    flush_in_progress: bool,
    #[serde(default)]
    total_events_flushed: u64,
}

pub use event_sink_canister::Event;

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

    pub fn info(&self) -> EventSinkClientInfo {
        let guard = self.inner.try_lock().unwrap();

        EventSinkClientInfo {
            event_sink_canister_id: guard.event_sink_canister_id,
            flush_delay: guard.flush_delay,
            max_batch_size: guard.max_batch_size as u32,
            events_pending: guard.events.len() as u32,
            flush_in_progress: guard.flush_in_progress,
            total_events_flushed: guard.total_events_flushed,
        }
    }
}

impl Client<NullRuntime> {
    pub fn null() -> Client<NullRuntime> {
        ClientBuilder::new(Principal::anonymous(), NullRuntime).build()
    }
}

pub struct EventSinkClientBuilder<R> {
    event_sink_canister_id: Principal,
    runtime: R,
    flush_delay: Option<Duration>,
    max_batch_size: Option<u32>,
    events: Vec<IdempotentEvent>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EventSinkClientInfo {
    pub event_sink_canister_id: Principal,
    pub flush_delay: Duration,
    pub max_batch_size: u32,
    pub events_pending: u32,
    pub flush_in_progress: bool,
    pub total_events_flushed: u64,
}

impl<R: Runtime + Send + 'static> ClientBuilder<R> {
    pub fn new(event_sink_canister_id: Principal, runtime: R) -> ClientBuilder<R> {
        ClientBuilder {
            event_sink_canister_id,
            runtime,
            flush_delay: None,
            max_batch_size: None,
            events: Vec::new(),
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

    pub fn with_events(mut self, events: Vec<IdempotentEvent>) -> Self {
        self.events = events;
        self
    }

    pub fn build(self) -> Client<R> {
        let flush_delay = self.flush_delay.unwrap_or(DEFAULT_FLUSH_DELAY);
        let max_batch_size = self.max_batch_size.unwrap_or(DEFAULT_MAX_BATCH_SIZE) as usize;
        let any_events = !self.events.is_empty();

        let client = Client {
            inner: Arc::new(Mutex::new(ClientInner::new(
                self.event_sink_canister_id,
                self.runtime,
                flush_delay,
                max_batch_size,
                self.events,
            ))),
        };
        if any_events {
            let guard = client.inner.try_lock().unwrap();
            client.process_events(guard, false);
        }
        client
    }
}

impl<R: Runtime + Send + 'static> Client<R> {
    pub fn push_event(&mut self, event: Event) {
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

    pub fn flush_batch(&self) {
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
            let event_sink_canister_id = guard.event_sink_canister_id;
            guard
                .runtime
                .flush(event_sink_canister_id, events.clone(), move |outcome| {
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
        let mut guard = self.inner.try_lock().unwrap();
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
        event_sink_canister_id: Principal,
        runtime: R,
        flush_delay: Duration,
        max_batch_size: usize,
        events: Vec<IdempotentEvent>,
    ) -> ClientInner<R> {
        ClientInner {
            event_sink_canister_id,
            runtime,
            flush_delay,
            max_batch_size,
            events,
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

impl<'de, R: Deserialize<'de>> Deserialize<'de> for Client<R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = ClientInner::deserialize(deserializer)?;

        Ok(Client {
            inner: Arc::new(Mutex::new(inner)),
        })
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
