use event_sink_canister::{Event, IdempotentEvent, TimestampMillis};
use std::mem;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

pub struct Client<R> {
    inner: Arc<Mutex<ClientInner<R>>>,
}

struct ClientInner<R> {
    runtime: R,
    flush_interval: Duration,
    next_flush_scheduled: Option<TimestampMillis>,
    events: Vec<IdempotentEvent>,
}

pub trait Runtime {
    fn schedule_flush<F: FnOnce() + 'static>(&mut self, delay: Duration, callback: F);
    fn flush<F: FnOnce() + 'static>(&mut self, events: Vec<IdempotentEvent>, trigger_retry: F);
    fn rng(&mut self) -> u128;
    fn now(&self) -> TimestampMillis;
}

impl<R> Client<R> {
    pub fn new(runtime: R, flush_interval: Duration) -> Client<R> {
        Client {
            inner: Arc::new(Mutex::new(ClientInner::new(runtime, flush_interval))),
        }
    }

    pub fn take_events(&mut self) -> Vec<IdempotentEvent> {
        mem::take(&mut self.inner.lock().unwrap().events)
    }
}

impl<R: Runtime + 'static> Client<R> {
    pub fn init_with_events(
        runtime: R,
        flush_interval: Duration,
        events: Vec<IdempotentEvent>,
    ) -> Client<R> {
        let client = Client::new(runtime, flush_interval);
        client.requeue_events(events);
        client
    }

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
            let flush_interval = guard.flush_interval;
            guard
                .runtime
                .schedule_flush(flush_interval, move || clone.flush_events());
            guard.next_flush_scheduled = Some(now + flush_interval.as_millis() as u64);
        }
    }

    fn flush_events(&self) {
        let mut guard = self.inner.lock().unwrap();
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
    pub fn new(runtime: R, flush_interval: Duration) -> ClientInner<R> {
        ClientInner {
            runtime,
            flush_interval,
            next_flush_scheduled: None,
            events: Vec::new(),
        }
    }
}
