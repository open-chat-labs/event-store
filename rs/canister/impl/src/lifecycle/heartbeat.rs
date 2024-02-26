use crate::state;
use ic_cdk::heartbeat;

#[heartbeat]
fn heartbeat() {
    state::mutate(|s| s.events.migrate(1000));
}
