use crate::model::events::Events;
use candid::Principal;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashSet;

thread_local! {
    static STATE: RefCell<Option<State>> = RefCell::default();
}

#[derive(Serialize, Deserialize)]
pub struct State {
    push_events_whitelist: HashSet<Principal>,
    read_events_whitelist: HashSet<Principal>,
    remove_events_whitelist: HashSet<Principal>,
    pub events: Events,
}

pub fn read<F: FnOnce(&State) -> R, R>(f: F) -> R {
    STATE.with_borrow(|state| f(state.as_ref().unwrap()))
}

pub fn mutate<F: FnOnce(&mut State) -> R, R>(f: F) -> R {
    STATE.with_borrow_mut(|state| f(state.as_mut().unwrap()))
}

impl State {
    pub fn can_caller_push_events(&self) -> bool {
        let caller = ic_cdk::caller();
        self.push_events_whitelist.contains(&caller)
    }

    pub fn can_caller_read_events(&self) -> bool {
        let caller = ic_cdk::caller();
        self.read_events_whitelist.contains(&caller)
    }

    pub fn can_caller_remove_events(&self) -> bool {
        let caller = ic_cdk::caller();
        self.read_events_whitelist.contains(&caller)
    }
}
