use crate::lifecycle::READER_WRITER_BUFFER_SIZE;
use crate::memory::get_upgrades_memory;
use crate::state;
use crate::state::State;
use ic_cdk::post_upgrade;
use ic_stable_structures::reader::{BufferedReader, Reader};
use serde::Deserialize;
use std::time::Duration;

#[post_upgrade]
fn post_upgrade() {
    let memory = get_upgrades_memory();
    let reader = BufferedReader::new(READER_WRITER_BUFFER_SIZE, Reader::new(&memory, 0));
    let mut deserializer = rmp_serde::Deserializer::new(reader);

    state::init(State::deserialize(&mut deserializer).unwrap());

    run_job_to_populate_integrations_data_if_required()
}

fn run_job_to_populate_integrations_data_if_required() {
    state::read(|s| {
        if let Some(next) = s.integrations_data().next_event_index() {
            if s.events().stats().latest_event_index > Some(next) {
                ic_cdk_timers::set_timer(Duration::ZERO, populate_integrations_data);
            }
        }
    });
}

fn populate_integrations_data() {
    state::mutate(|s| {
        if let Some(next) = s.integrations_data().next_event_index() {
            let events = s.events().get(next, 10_000);
            for event in events {
                s.integrations_data_mut().push_event(event);
            }
        }
    });
    run_job_to_populate_integrations_data_if_required();
}
