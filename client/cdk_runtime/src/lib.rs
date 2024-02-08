use event_sink_canister::{IdempotentEvent, PushEventsArgs, TimestampMillis};
use event_sink_client::Runtime;
use ic_cdk_timers::TimerId;
use ic_principal::Principal;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::Duration;

pub struct CdkRuntime {
    rng: StdRng,
    scheduled_flush_timer: Option<TimerId>,
}

impl CdkRuntime {
    fn clear_timer(&mut self) {
        if let Some(timer_id) = self.scheduled_flush_timer.take() {
            ic_cdk_timers::clear_timer(timer_id);
        };
    }
}

impl Runtime for CdkRuntime {
    fn schedule_flush<F: FnOnce() + Send + 'static>(&mut self, delay: Duration, callback: F) {
        self.clear_timer();
        self.scheduled_flush_timer = Some(ic_cdk_timers::set_timer(delay, callback));
    }

    fn flush<F: FnOnce() + Send + 'static>(
        &mut self,
        canister_id: Principal,
        events: Vec<IdempotentEvent>,
        trigger_retry: F,
    ) {
        self.clear_timer();
        ic_cdk::spawn(flush_async(canister_id, events, trigger_retry))
    }

    fn rng(&mut self) -> u128 {
        self.rng.gen()
    }

    fn now(&self) -> TimestampMillis {
        ic_cdk::api::time() / 1_000_000
    }
}

async fn flush_async<F: FnOnce()>(
    canister_id: Principal,
    events: Vec<IdempotentEvent>,
    trigger_retry: F,
) {
    if ic_cdk::call::<_, ()>(canister_id, "push_events", (PushEventsArgs { events },))
        .await
        .is_err()
    {
        trigger_retry();
    }
}

impl Default for CdkRuntime {
    fn default() -> Self {
        let mut seed = [0; 32];
        seed[..8].copy_from_slice(&ic_cdk::api::time().to_ne_bytes());

        CdkRuntime {
            rng: StdRng::from_seed(seed),
            scheduled_flush_timer: None,
        }
    }
}
