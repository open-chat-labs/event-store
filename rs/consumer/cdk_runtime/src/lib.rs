use event_store_canister::{EventsArgs, EventsResponse};
use event_store_consumer::Runtime;
use ic_cdk::call::{Call, CallFailed};
use ic_principal::Principal;
use serde::{Deserialize, Serialize};
use std::future::Future;

#[derive(Serialize, Deserialize, Default)]
pub struct CdkRuntime;

impl CdkRuntime {
    async fn events_async(
        &self,
        canister_id: Principal,
        args: EventsArgs,
    ) -> Result<EventsResponse, (i32, String)> {
        match Call::unbounded_wait(canister_id, "events")
            .with_arg(args)
            .await
        {
            Ok(response) => response
                .candid()
                .map_err(|e| (0, format!("Failed to deserialize response: {e}"))),
            Err(CallFailed::InsufficientLiquidCycleBalance(e)) => Err((0, e.to_string())),
            Err(CallFailed::CallPerformFailed(f)) => Err((0, f.to_string())),
            Err(CallFailed::CallRejected(r)) => {
                Err((r.raw_reject_code() as i32, r.reject_message().to_string()))
            }
        }
    }
}

impl Runtime for CdkRuntime {
    fn events(
        &self,
        canister_id: Principal,
        args: EventsArgs,
    ) -> impl Future<Output = Result<EventsResponse, (i32, String)>> + Send {
        self.events_async(canister_id, args)
    }
}
