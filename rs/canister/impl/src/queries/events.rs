use crate::guards::caller_can_read_events;
use crate::state;
use event_store_canister::{EventsArgs, EventsResponse};
use ic_cdk::query;

#[query(guard = "caller_can_read_events")]
fn events(args: EventsArgs) -> EventsResponse {
    state::read(|s| {
        let events = s.events().get(args.start, args.length);
        let stats = s.events().stats();

        EventsResponse {
            events,
            latest_event_index: stats.latest_event_index,
        }
    })
}
