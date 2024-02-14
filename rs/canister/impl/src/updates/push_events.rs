use crate::guards::caller_can_push_events;
use crate::{env, state};
use event_sink_canister::PushEventsArgs;
use ic_cdk::update;

#[update(guard = "caller_can_push_events")]
fn push_events(args: PushEventsArgs) {
    let now = env::time();

    state::mutate(|s| {
        for event in args.events {
            if s.event_deduper.try_push(event.idempotency_key, now) {
                s.events.push(event);
            }
        }
    });
}
