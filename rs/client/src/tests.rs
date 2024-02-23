use crate::{ClientBuilder, EventBuilder, FlushOutcome, Runtime};
use event_sink_canister::{IdempotentEvent, TimestampMillis};
use ic_principal::Principal;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;
use test_case::test_case;

#[test_case(true)]
#[test_case(false)]
fn batch_flushed_when_max_batch_size_reached(flush_synchronously: bool) {
    let runtime = TestRuntime::new(flush_synchronously);
    let mut client = ClientBuilder::new(Principal::anonymous(), runtime.clone())
        .with_max_batch_size(5)
        .build();

    for i in 0..10 {
        for _ in 0..5 {
            client.push(EventBuilder::new(i, 0).build());
        }
        thread::sleep(Duration::from_millis(10));
        assert_eq!(client.info().events_pending, 0);
        assert_eq!(runtime.inner().flush_invocations, i + 1);
        runtime.tick();
    }
}

#[test_case(true)]
#[test_case(false)]
fn batch_flushed_when_flush_delay_reached(flush_synchronously: bool) {
    let runtime = TestRuntime::new(flush_synchronously);
    let mut client = ClientBuilder::new(Principal::anonymous(), runtime.clone())
        .with_flush_delay(Duration::from_secs(5))
        .build();

    for i in 0..10 {
        for _ in 0..5 {
            client.push(EventBuilder::new(i, 0).build());
        }
        runtime.inner().timestamp += 4999;
        runtime.tick();
        runtime.tick();
        thread::sleep(Duration::from_millis(10));
        assert_eq!(client.info().events_pending, 5);
        assert_eq!(runtime.inner().flush_invocations, i);
        runtime.inner().timestamp += 1;
        runtime.tick();
        runtime.tick();
        thread::sleep(Duration::from_millis(10));
        assert_eq!(client.info().events_pending, 0);
        assert_eq!(runtime.inner().flush_invocations, i + 1);
    }
}

#[derive(Default, Clone)]
struct TestRuntime {
    inner: Arc<Mutex<TestRuntimeInner>>,
    flush_synchronously: bool,
}

impl TestRuntime {
    fn new(flush_synchronously: bool) -> TestRuntime {
        TestRuntime {
            flush_synchronously,
            ..Default::default()
        }
    }

    fn inner(&self) -> MutexGuard<TestRuntimeInner> {
        self.inner.try_lock().unwrap()
    }
}

#[derive(Default)]
struct TestRuntimeInner {
    timestamp: TimestampMillis,
    rng: u128,
    flush_outcome: FlushOutcome,
    schedule_flush_invocations: u32,
    callback_due_at: Option<TimestampMillis>,
    callback: Option<Box<dyn FnOnce() + Send + 'static>>,
    flush_invocations: u32,
    rng_invocations: u32,
    now_invocations: u32,
}

impl Runtime for TestRuntime {
    fn schedule_flush<F: FnOnce() + Send + 'static>(&mut self, delay: Duration, callback: F) {
        let mut guard = self.inner();
        guard.schedule_flush_invocations += 1;
        guard.callback_due_at = Some(guard.timestamp + delay.as_millis() as u64);
        guard.callback = Some(Box::new(callback));
    }

    fn flush<F: FnOnce(FlushOutcome) + Send + 'static>(
        &mut self,
        _event_sync_canister_id: Principal,
        _events: Vec<IdempotentEvent>,
        on_complete: F,
    ) {
        let mut guard = self.inner();
        guard.flush_invocations += 1;
        let outcome = guard.flush_outcome;

        if self.flush_synchronously {
            guard.callback_due_at = None;
            guard.callback = None;
            on_complete(outcome);
        } else {
            guard.callback_due_at = Some(guard.timestamp);
            guard.callback = Some(Box::new(move || on_complete(outcome)));
        }
    }

    fn rng(&mut self) -> u128 {
        let mut guard = self.inner();
        guard.rng_invocations += 1;
        guard.rng
    }

    fn now(&self) -> TimestampMillis {
        let mut guard = self.inner();
        guard.now_invocations += 1;
        guard.timestamp
    }
}

impl TestRuntime {
    fn tick(&self) {
        if let Some(callback) = self.take_callback_if_due() {
            callback()
        }
    }

    fn take_callback_if_due(&self) -> Option<Box<dyn FnOnce() + Send + 'static>> {
        let mut guard = self.inner();
        guard
            .callback_due_at
            .filter(|ts| *ts <= guard.timestamp)
            .take()
            .and_then(|_| guard.callback.take())
    }
}
