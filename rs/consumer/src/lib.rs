use event_store_canister::{EventsArgs, EventsResponse};
use ic_principal::Principal;

pub struct EventStoreClient<R> {
    event_store_canister_id: Principal,
    runtime: R,
}

impl<R> EventStoreClient<R> {
    pub fn new(event_store_canister_id: Principal, runtime: R) -> Self {
        Self {
            event_store_canister_id,
            runtime,
        }
    }
}

impl<R: Runtime> EventStoreClient<R> {
    pub async fn events(&self, start: u64, length: u64) -> Result<EventsResponse, (i32, String)> {
        self.runtime
            .events(self.event_store_canister_id, EventsArgs { start, length })
            .await
    }
}

pub trait Runtime {
    fn events(
        &self,
        canister_id: Principal,
        args: EventsArgs,
    ) -> impl std::future::Future<Output = Result<EventsResponse, (i32, String)>> + Send;
}
