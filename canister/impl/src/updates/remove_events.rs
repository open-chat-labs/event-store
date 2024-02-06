use crate::guards::caller_can_remove_events;
use crate::state;
use event_sink_canister::{RemoveEventsArgs, RemoveEventsResponse};
use ic_cdk::update;

#[update(guard = "caller_can_remove_events")]
fn remove_events(args: RemoveEventsArgs) -> RemoveEventsResponse {
    state::mutate(|s| {
        s.events.remove(args.up_to_inclusive);
        let stats = s.events.stats();

        RemoveEventsResponse {
            latest_event_index: stats.latest_event_index,
            earliest_event_index_stored: stats.earliest_event_index_stored,
        }
    })
}
