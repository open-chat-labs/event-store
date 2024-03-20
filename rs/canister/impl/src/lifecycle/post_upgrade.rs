use crate::lifecycle::READER_WRITER_BUFFER_SIZE;
use crate::memory::{
    get_events_v2_data_memory, get_events_v2_index_memory, get_num_to_string_v2_data_memory,
    get_num_to_string_v2_index_memory, get_string_to_num_v2_map_memory, get_upgrades_memory,
};
use crate::state;
use crate::state::State;
use ic_cdk::post_upgrade;
use ic_stable_structures::reader::{BufferedReader, Reader};
use ic_stable_structures::Memory;
use serde::Deserialize;

#[post_upgrade]
fn post_upgrade() {
    get_events_v2_index_memory().write(0, &[0, 0, 0]);
    get_events_v2_data_memory().write(0, &[0, 0, 0]);
    get_string_to_num_v2_map_memory().write(0, &[0, 0, 0]);
    get_num_to_string_v2_index_memory().write(0, &[0, 0, 0]);
    get_num_to_string_v2_data_memory().write(0, &[0, 0, 0]);

    let memory = get_upgrades_memory();
    let reader = BufferedReader::new(READER_WRITER_BUFFER_SIZE, Reader::new(&memory, 0));
    let mut deserializer = rmp_serde::Deserializer::new(reader);

    state::init(State::deserialize(&mut deserializer).unwrap());
}
