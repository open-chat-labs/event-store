use crate::state;
use crate::state::State;
use event_store_canister::InitArgs;
use ic_cdk::init;
use std::time::Duration;

#[init]
fn init(args: InitArgs) {
    state::init(State::new(
        args.push_events_whitelist.into_iter().collect(),
        args.read_events_whitelist.into_iter().collect(),
        args.time_granularity,
    ));

    ic_cdk_timers::set_timer(Duration::ZERO, async {
        let salt: [u8; 32] = ic_cdk_management_canister::raw_rand()
            .await
            .unwrap()
            .try_into()
            .unwrap();

        state::mutate(|s| s.set_salt(salt));
    });
}
