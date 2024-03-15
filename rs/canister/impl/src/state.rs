use crate::env;
use crate::model::events::Events;
use crate::model::salt::Salt;
use candid::Principal;
use event_store_canister::{IdempotentEvent, TimestampMillis, WhitelistedPrincipals};
use event_store_utils::EventDeduper;
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
    #[serde(skip)]
    events: Events,
    event_deduper: EventDeduper,
    salt: Salt,
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
    ) -> State {
        State {
            push_events_whitelist,
            read_events_whitelist,
            events: Events::default(),
            event_deduper: EventDeduper::default(),
            salt: Salt::default(),
        }
    }

    pub fn can_caller_push_events(&self) -> bool {
        let caller = env::caller();
        self.push_events_whitelist.contains(&caller)
    }

    pub fn can_caller_read_events(&self) -> bool {
        let caller = env::caller();
        self.read_events_whitelist.contains(&caller)
    }

    pub fn whitelisted_principals(&self) -> WhitelistedPrincipals {
        WhitelistedPrincipals {
            push: self.push_events_whitelist.iter().copied().collect(),
            read: self.read_events_whitelist.iter().copied().collect(),
        }
    }

    pub fn events(&self) -> &Events {
        &self.events
    }

    pub fn set_salt(&mut self, salt: [u8; 32]) {
        self.salt.set(salt);
    }

    pub fn push_event(&mut self, event: IdempotentEvent, now: TimestampMillis) {
        if self.event_deduper.try_push(event.idempotency_key, now) {
            self.events.push(event, self.salt.get());
        }
    }
}
