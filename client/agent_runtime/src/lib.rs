use candid::Principal;
use event_sink_canister::{IdempotentEvent, PushEventsArgs, TimestampMillis};
use event_sink_client::Runtime;
use ic_agent::Agent;
use rand::random;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio_util::sync::CancellationToken;

pub struct AgentRuntime {
    canister_id: Principal,
    agent: Agent,
    scheduler_task_cancellation_token: Option<CancellationToken>,
}

impl AgentRuntime {
    pub fn new(canister_id: Principal, agent: Agent) -> AgentRuntime {
        AgentRuntime {
            canister_id,
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

    fn flush<F: FnOnce() + Send + 'static>(
        &mut self,
        events: Vec<IdempotentEvent>,
        trigger_retry: F,
    ) {
        self.cancel_scheduler_task();
        let canister_id = self.canister_id;
        let agent = self.agent.clone();

        tokio::spawn(async move { flush_async(canister_id, agent, events, trigger_retry).await });
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

async fn flush_async<F: FnOnce() + Send + 'static>(
    canister_id: Principal,
    agent: Agent,
    events: Vec<IdempotentEvent>,
    trigger_retry: F,
) {
    if agent
        .update(&canister_id, "push_events".to_string())
        .with_arg(candid::encode_one(PushEventsArgs { events }).unwrap())
        .call_and_wait()
        .await
        .is_err()
    {
        trigger_retry()
    }
}
