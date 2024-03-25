use crate::memory::{
    get_num_to_string_data_memory, get_num_to_string_index_memory, get_string_to_num_map_memory,
    Memory,
};
use ic_stable_structures::{StableBTreeMap, StableLog};

pub struct StringToNumMap {
    string_to_num: StableBTreeMap<String, u32, Memory>,
    num_to_string: StableLog<String, Memory, Memory>,
}

impl StringToNumMap {
    pub fn convert_to_num(&mut self, string: String) -> u32 {
        if let Some(i) = self.string_to_num.get(&string) {
            i
        } else {
            let i = self.num_to_string.len() as u32;
            self.num_to_string.append(&string).unwrap();
            self.string_to_num.insert(string, i);
            i
        }
    }

    pub fn convert_to_string(&self, num: u32) -> Option<String> {
        self.num_to_string.get(num as u64)
    }
}

impl Default for StringToNumMap {
    fn default() -> Self {
        StringToNumMap {
            string_to_num: init_string_to_num(),
            num_to_string: init_num_to_string(),
        }
    }
}

fn init_string_to_num() -> StableBTreeMap<String, u32, Memory> {
    StableBTreeMap::init(get_string_to_num_map_memory())
}

fn init_num_to_string() -> StableLog<String, Memory, Memory> {
    StableLog::init(
        get_num_to_string_index_memory(),
        get_num_to_string_data_memory(),
    )
    .unwrap()
}
