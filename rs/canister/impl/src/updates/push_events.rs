use crate::guards::caller_can_push_events;
use crate::{env, state};
use event_store_canister::{Anonymizable, IdempotentEvent, PushEventsArgs, PushEventsArgsPrevious};
use ic_cdk::update;

#[update(guard = "caller_can_push_events")]
fn push_events(args: PushEventsArgsPrevious) {
    let now = env::time();

    state::mutate(|s| {
        for event in args.events {
            s.push_event(
                IdempotentEvent {
                    idempotency_key: event.idempotency_key,
                    name: event.name,
                    timestamp: event.timestamp,
                    user: event.user.map(Anonymizable::Public),
                    source: event.source.map(Anonymizable::Public),
                    payload: event.payload,
                },
                now,
            );
        }
    });
}

#[update(guard = "caller_can_push_events")]
fn push_events_v2(args: PushEventsArgs) {
    let now = env::time();

    state::mutate(|s| {
        for event in args.events {
            s.push_event(event, now);
        }
    });
}
