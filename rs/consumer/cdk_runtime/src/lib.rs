use event_store_canister::{EventsArgs, EventsResponse};
use event_store_consumer::Runtime;
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
        match ic_cdk::call(canister_id, "events", (args,)).await {
            Ok((response,)) => Ok(response),
            Err((code, msg)) => Err((code as i32, msg)),
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
