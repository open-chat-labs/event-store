use crate::guards::caller_can_push_events;
use crate::{env, state};
use event_store_canister::PushEventsArgs;
use ic_cdk::update;

#[update(guard = "caller_can_push_events")]
fn push_events(args: PushEventsArgs) {
    push_events_inner(args)
}

#[update(guard = "caller_can_push_events")]
fn push_events_v2(args: PushEventsArgs) {
    push_events_inner(args)
}

fn push_events_inner(args: PushEventsArgs) {
    let now = env::time();

    state::mutate(|s| {
        for event in args.events {
            s.push_event(event, now);
        }
    });
}
