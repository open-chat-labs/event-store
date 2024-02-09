use event_sink_canister::{IdempotentEvent, PushEventsArgs, TimestampMillis};
use event_sink_client::{
    FlushOutcome, Runtime, FLUSH_OUTCOME_FAILED_SHOULD_RETRY, FLUSH_OUTCOME_SUCCESS,
};
use ic_agent::Agent;
use ic_principal::Principal;
use rand::random;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio_util::sync::CancellationToken;

pub struct AgentRuntime {
    agent: Agent,
    scheduler_task_cancellation_token: Option<CancellationToken>,
}

impl AgentRuntime {
    pub fn new(agent: Agent) -> AgentRuntime {
        AgentRuntime {
            agent,
            scheduler_task_cancellation_token: None,
        }
    }

    fn cancel_scheduler_task(&mut self) {
        if let Some(token) = self.scheduler_task_cancellation_token.take() {
            token.cancel()
        }
    }
}

impl Runtime for AgentRuntime {
    fn schedule_flush<F: FnOnce() + Send + 'static>(&mut self, delay: Duration, callback: F) {
        self.cancel_scheduler_task();
        let token = CancellationToken::new();
        self.scheduler_task_cancellation_token = Some(token.clone());

        tokio::spawn(async move {
            tokio::select! {
                _ = token.cancelled() => {}
                _ = tokio::time::sleep(delay) => callback()
            }
        });
    }

    fn flush<F: FnOnce(FlushOutcome) + Send + 'static>(
        &mut self,
        canister_id: Principal,
        events: Vec<IdempotentEvent>,
        on_complete: F,
    ) {
        self.cancel_scheduler_task();
        let agent = self.agent.clone();

        tokio::spawn(async move { flush_async(canister_id, agent, events, on_complete).await });
    }

    fn rng(&mut self) -> u128 {
        random()
    }

    fn now(&self) -> TimestampMillis {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

async fn flush_async<F: FnOnce(FlushOutcome) + Send + 'static>(
    canister_id: Principal,
    agent: Agent,
    events: Vec<IdempotentEvent>,
    on_complete: F,
) {
    if agent
        .update(&canister_id, "push_events".to_string())
        .with_arg(candid::encode_one(PushEventsArgs { events }).unwrap())
        .call_and_wait()
        .await
        .is_err()
    {
        on_complete(FLUSH_OUTCOME_FAILED_SHOULD_RETRY)
    } else {
        on_complete(FLUSH_OUTCOME_SUCCESS)
    }
}
