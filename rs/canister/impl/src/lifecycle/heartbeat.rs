use crate::state;
use ic_cdk::heartbeat;

#[heartbeat]
fn heartbeat() {
    state::mutate(|s| s.migrate_events(10000));
}
