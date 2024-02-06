use crate::state;
use crate::state::State;
use event_sink_canister::init::InitArgs;
use ic_cdk::init;

#[init]
fn init(args: InitArgs) {
    state::init(State::new(
        args.push_events_whitelist.into_iter().collect(),
        args.read_events_whitelist.into_iter().collect(),
        args.remove_events_whitelist.into_iter().collect(),
    ));
}
