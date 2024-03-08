use event_store_canister::{EventsArgs, EventsResponse};
use ic_principal::Principal;

pub struct EventStoreClient<R> {
    event_store_canister_id: Principal,
    runtime: R,
    synced_up_to: Option<u64>,
    batch_size: u64,
}

pub struct EventStoreClientBuilder<R> {
    event_store_canister_id: Principal,
    runtime: R,
    synced_up_to: Option<u64>,
    batch_size: Option<u64>,
}

impl<R: Runtime> EventStoreClient<R> {
    pub async fn next_batch(&mut self) -> Result<EventsResponse, (i32, String)> {
        let response = self
            .runtime
            .events(
                self.event_store_canister_id,
                EventsArgs {
                    start: self.synced_up_to.map_or(0, |i| i + 1),
                    length: self.batch_size,
                },
            )
            .await?;

        if let Some(event) = response.events.last() {
            self.synced_up_to = Some(event.index);
        }

        Ok(response)
    }
}

impl<R> EventStoreClientBuilder<R> {
    pub fn new(event_store_canister_id: Principal, runtime: R) -> Self {
        Self {
            event_store_canister_id,
            runtime,
            synced_up_to: None,
            batch_size: None,
        }
    }

    pub fn set_synced_up_to(mut self, synced_up_to: u64) -> Self {
        self.synced_up_to = Some(synced_up_to);
        self
    }

    pub fn with_batch_size(mut self, batch_size: u64) -> Self {
        self.batch_size = Some(batch_size);
        self
    }

    pub fn build(self) -> EventStoreClient<R> {
        EventStoreClient {
            event_store_canister_id: self.event_store_canister_id,
            runtime: self.runtime,
            synced_up_to: self.synced_up_to,
            batch_size: self.batch_size.unwrap_or(1000),
        }
    }
}

pub trait Runtime {
    fn events(
        &self,
        canister_id: Principal,
        args: EventsArgs,
    ) -> impl std::future::Future<Output = Result<EventsResponse, (i32, String)>> + Send;
}
