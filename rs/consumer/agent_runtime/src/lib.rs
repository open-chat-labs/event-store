use event_store_canister::{EventsArgs, EventsResponse};
use event_store_consumer::Runtime;
use ic_agent::{Agent, AgentError};
use ic_principal::Principal;
use std::future::Future;

pub struct AgentRuntime {
    agent: Agent,
}

impl AgentRuntime {
    pub fn new(agent: Agent) -> Self {
        Self { agent }
    }

    async fn events_async(
        &self,
        canister_id: Principal,
        args: EventsArgs,
    ) -> Result<EventsResponse, (i32, String)> {
        match self
            .agent
            .query(&canister_id, "events")
            .with_arg(candid::encode_one(args).unwrap())
            .call()
            .await
        {
            Ok(response) => Ok(candid::decode_one(&response).unwrap()),
            Err(AgentError::CertifiedReject(error)) | Err(AgentError::UncertifiedReject(error)) => {
                Err((error.reject_code as i32, error.reject_message))
            }
            Err(error) => Err((0, error.to_string())),
        }
    }
}

impl Runtime for AgentRuntime {
    fn events(
        &self,
        canister_id: Principal,
        args: EventsArgs,
    ) -> impl Future<Output = Result<EventsResponse, (i32, String)>> + Send {
        self.events_async(canister_id, args)
    }
}
