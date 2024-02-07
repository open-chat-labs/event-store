use candid::Principal;
use event_sink_canister::{IdempotentEvent, PushEventsArgs, TimestampMillis};
use event_sink_client::Runtime;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::Duration;

#[derive(Clone)]
pub struct CdkRuntime {
    canister_id: Principal,
    rng: StdRng,
}

impl CdkRuntime {
    pub fn new(canister_id: Principal) -> CdkRuntime {
        let mut seed = [0; 32];
        seed[..8].copy_from_slice(&ic_cdk::api::time().to_ne_bytes());

        CdkRuntime {
            canister_id,
            rng: StdRng::from_seed(seed),
        }
    }

    async fn flush_async<F: FnOnce()>(self, events: Vec<IdempotentEvent>, trigger_retry: F) {
        if ic_cdk::call::<_, ()>(
            self.canister_id,
            "push_events",
            (PushEventsArgs { events },),
        )
        .await
        .is_err()
        {
            trigger_retry();
        }
    }
}

impl Runtime for CdkRuntime {
    fn schedule_callback<F: FnOnce() + 'static>(&self, delay: Duration, callback: F) {
        ic_cdk_timers::set_timer(delay, callback);
    }

    fn flush<F: FnOnce() + 'static>(&self, events: Vec<IdempotentEvent>, trigger_retry: F) {
        let clone = self.clone();
        ic_cdk::spawn(clone.flush_async(events, trigger_retry))
    }

    fn rng(&mut self) -> u128 {
        self.rng.gen()
    }

    fn now(&self) -> TimestampMillis {
        ic_cdk::api::time() / 1_000_000
    }
}
