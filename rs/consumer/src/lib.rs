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
    pub async fn events(&self, args: EventsArgs) -> Result<EventsResponse, (i32, String)> {
        self.runtime
            .events(self.event_store_canister_id, args)
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
