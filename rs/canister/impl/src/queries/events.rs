use crate::guards::caller_can_read_events;
use crate::state;
use event_store_canister::{EventsArgs, EventsResponse};
use ic_cdk::query;

#[query(guard = "caller_can_read_events")]
fn events(args: EventsArgs) -> EventsResponse {
    state::read(|s| {
        let stats = s.events().stats();
        let stats_v2 = s.events_v2().stats();
        let (events, is_v2) = if stats.latest_event_index > stats_v2.latest_event_index {
            (s.events().get(args.start, args.length), false)
        } else {
            (s.events_v2().get(args.start, args.length), true)
        };

        EventsResponse {
            events,
            latest_event_index: stats.latest_event_index,
            latest_event_index_v2: stats_v2.latest_event_index,
            is_v2,
        }
    })
}
