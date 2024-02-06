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

const STATE_ALREADY_INITIALIZED: &str = "State has already been initialized";
const STATE_NOT_INITIALIZED: &str = "State has not been initialized";

pub fn init(state: State) {
    STATE.with_borrow_mut(|s| {
        if s.is_some() {
            panic!("{}", STATE_ALREADY_INITIALIZED);
        } else {
            *s = Some(state);
        }
    })
}

pub fn read<F: FnOnce(&State) -> R, R>(f: F) -> R {
    STATE.with_borrow(|s| f(s.as_ref().expect(STATE_NOT_INITIALIZED)))
}

pub fn mutate<F: FnOnce(&mut State) -> R, R>(f: F) -> R {
    STATE.with_borrow_mut(|s| f(s.as_mut().expect(STATE_NOT_INITIALIZED)))
}

pub fn take() -> State {
    STATE.take().expect(STATE_NOT_INITIALIZED)
}

impl State {
    pub fn new(
        push_events_whitelist: HashSet<Principal>,
        read_events_whitelist: HashSet<Principal>,
        remove_events_whitelist: HashSet<Principal>,
    ) -> State {
        State {
            push_events_whitelist,
            read_events_whitelist,
            remove_events_whitelist,
            events: Events::default(),
        }
    }

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
        self.remove_events_whitelist.contains(&caller)
    }
}
