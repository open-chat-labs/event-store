use event_store_canister::PushEventsArgs;
use event_store_producer::{
    FLUSH_OUTCOME_FAILED_SHOULD_RETRY, FLUSH_OUTCOME_SUCCESS, FlushOutcome, IdempotentEvent,
    Runtime, TimestampMillis,
};
use ic_cdk::call::Call;
use ic_cdk_timers::TimerId;
use ic_principal::Principal;
#[cfg(not(target_arch = "wasm32"))]
use rand::random;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::time::Duration;
use tracing::{error, trace};

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
        self.scheduled_flush_timer =
            Some(ic_cdk_timers::set_timer(delay, async move { callback() }));
    }

    fn flush<F: FnOnce(FlushOutcome) + Send + 'static>(
        &mut self,
        canister_id: Principal,
        events: Vec<IdempotentEvent>,
        on_complete: F,
    ) {
        self.clear_timer();
        ic_cdk::futures::spawn(flush_async(canister_id, events, on_complete))
    }

    fn rng(&mut self) -> u128 {
        self.rng.random()
    }

    fn now(&self) -> TimestampMillis {
        ic_cdk::api::time() / 1_000_000
    }
}

async fn flush_async<F: FnOnce(FlushOutcome)>(
    canister_id: Principal,
    events: Vec<IdempotentEvent>,
    on_complete: F,
) {
    let events_len = events.len();
    if let Err(error) = Call::unbounded_wait(canister_id, "push_events")
        .with_arg(PushEventsArgs { events })
        .await
    {
        on_complete(FLUSH_OUTCOME_FAILED_SHOULD_RETRY);
        error!(%canister_id, events = events_len, ?error, "Failed to call 'push_events'");
    } else {
        on_complete(FLUSH_OUTCOME_SUCCESS);
        trace!(%canister_id, events = events_len, "Successfully called `push_events`");
    }
}

impl Default for CdkRuntime {
    fn default() -> Self {
        CdkRuntime {
            rng: StdRng::from_seed(rng_seed()),
            scheduled_flush_timer: None,
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn rng_seed() -> [u8; 32] {
    // The canister id can be up to 29 bytes, so cap it at 24 to ensure it can never overlap with
    // the timestamp which occupies the final 8 bytes
    let mut seed = [0; 32];
    let canister_id = ic_cdk::api::canister_self();
    let id_bytes = canister_id.as_slice();
    let id_len = id_bytes.len().min(24);
    seed[..id_len].copy_from_slice(&id_bytes[..id_len]);
    seed[24..].copy_from_slice(&ic_cdk::api::time().to_be_bytes());
    seed
}

#[cfg(not(target_arch = "wasm32"))]
fn rng_seed() -> [u8; 32] {
    random()
}

impl Serialize for CdkRuntime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_unit()
    }
}

impl<'de> Deserialize<'de> for CdkRuntime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_unit(serde::de::IgnoredAny)?;
        Ok(CdkRuntime::default())
    }
}
