use crate::guards::caller_can_push_events;
use crate::state;
use event_sink_canister::PushEventsArgs;
use ic_cdk::update;

#[update(guard = "caller_can_push_events")]
fn push_events(args: PushEventsArgs) {
    state::mutate(|s| {
        for event in args.events {
            s.events.push(event);
        }
    });
}
