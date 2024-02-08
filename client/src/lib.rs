use event_sink_canister::{IdempotentEvent, TimestampMillis};
use ic_principal::Principal;
use std::mem;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

const DEFAULT_FLUSH_DELAY: Duration = Duration::from_secs(300);
const DEFAULT_MAX_BATCH_SIZE: u32 = 1000;

pub struct Client<R> {
    inner: Arc<Mutex<ClientInner<R>>>,
}

struct ClientInner<R> {
    event_sink_canister_id: Principal,
    runtime: R,
    flush_delay: Duration,
    max_batch_size: usize,
    next_flush_scheduled: Option<TimestampMillis>,
    events: Vec<IdempotentEvent>,
}

pub use event_sink_canister::Event;

pub trait Runtime {
    fn schedule_flush<F: FnOnce() + Send + 'static>(&mut self, delay: Duration, callback: F);
    fn flush<F: FnOnce() + Send + 'static>(
        &mut self,
        event_sync_canister_id: Principal,
        events: Vec<IdempotentEvent>,
        trigger_retry: F,
    );
    fn rng(&mut self) -> u128;
    fn now(&self) -> TimestampMillis;
}

impl<R> Client<R> {
    pub fn take_events(&mut self) -> Vec<IdempotentEvent> {
        mem::take(&mut self.inner.lock().unwrap().events)
    }
}

pub struct ClientBuilder<R> {
    event_sink_canister_id: Principal,
    runtime: R,
    flush_delay: Option<Duration>,
    max_batch_size: Option<u32>,
    events: Vec<IdempotentEvent>,
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
        let client = Client {
            inner: Arc::new(Mutex::new(ClientInner::new(
                self.event_sink_canister_id,
                self.runtime,
                flush_delay,
                max_batch_size,
            ))),
        };
        if !self.events.is_empty() {
            client.requeue_events(self.events);
        }
        client
    }
}

impl<R: Runtime + Send + 'static> Client<R> {
    pub fn push_event(&mut self, event: Event) {
        let mut guard = self.inner.lock().unwrap();
        let idempotency_key = guard.runtime.rng();
        guard.events.push(IdempotentEvent {
            idempotency_key,
            name: event.name,
            timestamp: event.timestamp,
            user: event.user,
            source: event.source,
            payload: event.payload,
        });
        self.post_events_added(guard, true);
    }

    pub fn flush_batch(&self) {
        let mut guard = self.inner.lock().unwrap();
        guard.next_flush_scheduled = None;

        if !guard.events.is_empty() {
            let max_batch_size = guard.max_batch_size;

            let events = if guard.events.len() < max_batch_size {
                mem::take(&mut guard.events)
            } else {
                guard.events.drain(..max_batch_size).collect()
            };

            let clone = self.clone();
            let event_sink_canister_id = guard.event_sink_canister_id;
            guard
                .runtime
                .flush(event_sink_canister_id, events.clone(), move || {
                    clone.requeue_events(events)
                });
        }
    }

    pub fn queue_len(&self) -> usize {
        self.inner.lock().unwrap().events.len()
    }

    fn requeue_events(&self, events: Vec<IdempotentEvent>) {
        let mut guard = self.inner.lock().unwrap();
        guard.events.extend(events);
        self.post_events_added(guard, false);
    }

    fn post_events_added(&self, guard: MutexGuard<ClientInner<R>>, can_flush_immediately: bool) {
        let max_batch_size_reached = guard.events.len() >= guard.max_batch_size;
        if max_batch_size_reached {
            if can_flush_immediately {
                self.flush_batch();
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
    ) -> ClientInner<R> {
        ClientInner {
            event_sink_canister_id,
            runtime,
            flush_delay,
            max_batch_size,
            next_flush_scheduled: None,
            events: Vec::new(),
        }
    }
}
