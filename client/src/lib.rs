use event_sink_canister::{IdempotentEvent, TimestampMillis};
use std::mem;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

const DEFAULT_FLUSH_DELAY: Duration = Duration::from_secs(300);

pub struct Client<R> {
    inner: Arc<Mutex<ClientInner<R>>>,
}

struct ClientInner<R> {
    runtime: R,
    flush_delay: Duration,
    next_flush_scheduled: Option<TimestampMillis>,
    events: Vec<IdempotentEvent>,
}

pub use event_sink_canister::Event;

pub trait Runtime {
    fn schedule_flush<F: FnOnce() + Send + 'static>(&mut self, delay: Duration, callback: F);
    fn flush<F: FnOnce() + Send + 'static>(
        &mut self,
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
    runtime: R,
    flush_delay: Option<Duration>,
    events: Vec<IdempotentEvent>,
}

impl<R: Runtime + Send + 'static> ClientBuilder<R> {
    pub fn new(runtime: R) -> ClientBuilder<R> {
        ClientBuilder {
            runtime,
            flush_delay: None,
            events: Vec::new(),
        }
    }

    pub fn with_flush_delay(mut self, duration: Duration) -> Self {
        self.flush_delay = Some(duration);
        self
    }

    pub fn with_events(mut self, events: Vec<IdempotentEvent>) -> Self {
        self.events = events;
        self
    }

    pub fn build(self) -> Client<R> {
        let flush_delay = self.flush_delay.unwrap_or(DEFAULT_FLUSH_DELAY);
        let client = Client {
            inner: Arc::new(Mutex::new(ClientInner::new(self.runtime, flush_delay))),
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
            payload: event.payload,
        });
        self.schedule_flush_if_required(guard);
    }

    fn requeue_events(&self, events: Vec<IdempotentEvent>) {
        let mut guard = self.inner.lock().unwrap();
        guard.events.extend(events);
        self.schedule_flush_if_required(guard);
    }

    fn schedule_flush_if_required(&self, mut guard: MutexGuard<ClientInner<R>>) {
        if guard.next_flush_scheduled.is_none() {
            let clone = self.clone();
            let now = guard.runtime.now();
            let flush_delay = guard.flush_delay;
            guard
                .runtime
                .schedule_flush(flush_delay, move || clone.flush_events());
            guard.next_flush_scheduled = Some(now + flush_delay.as_millis() as u64);
        }
    }

    fn flush_events(&self) {
        let mut guard = self.inner.lock().unwrap();
        guard.next_flush_scheduled = None;
        let events = mem::take(&mut guard.events);

        if !events.is_empty() {
            let clone = self.clone();
            guard
                .runtime
                .flush(events.clone(), move || clone.requeue_events(events));
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
    pub fn new(runtime: R, flush_delay: Duration) -> ClientInner<R> {
        ClientInner {
            runtime,
            flush_delay,
            next_flush_scheduled: None,
            events: Vec::new(),
        }
    }
}
