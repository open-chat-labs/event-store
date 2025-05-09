use crate::env;
use crate::model::events::Events;
use crate::model::integrations_data::IntegrationsData;
use crate::model::salt::Salt;
use candid::Principal;
use event_store_canister::WhitelistedPrincipals;
use event_store_types::{IdempotentEvent, Milliseconds, TimestampMillis};
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
    time_granularity: Option<Milliseconds>,
    #[serde(skip)]
    events: Events,
    event_deduper: EventDeduper,
    #[serde(default)]
    integrations_data: IntegrationsData,
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
        time_granularity: Option<Milliseconds>,
    ) -> State {
        State {
            push_events_whitelist,
            read_events_whitelist,
            time_granularity,
            events: Events::default(),
            event_deduper: EventDeduper::default(),
            integrations_data: IntegrationsData::default(),
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

    pub fn push_event(&mut self, mut event: IdempotentEvent, now: TimestampMillis) {
        if self.event_deduper.try_push(event.idempotency_key, now) {
            if let Some(granularity) = self.time_granularity {
                event.timestamp = event
                    .timestamp
                    .saturating_sub(event.timestamp % granularity);
            }

            let indexed_event = self.events.push(event, self.salt.get());

            self.integrations_data.push_event(indexed_event);
        }
    }

    #[allow(dead_code)]
    pub fn integrations_data(&self) -> &IntegrationsData {
        &self.integrations_data
    }

    #[allow(dead_code)]
    pub fn integrations_data_mut(&mut self) -> &mut IntegrationsData {
        &mut self.integrations_data
    }
}
