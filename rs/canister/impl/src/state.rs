use crate::env;
use crate::model::events::Events;
use crate::model::salt::Salt;
use candid::Principal;
use event_store_canister::{
    AnonymizationInitConfig, IdempotentEvent, TimestampMillis, WhitelistedPrincipals,
};
use event_store_utils::EventDeduper;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Write;

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
    anonymization_config: AnonymizationConfig,
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
        anonymization_config: AnonymizationInitConfig,
    ) -> State {
        State {
            push_events_whitelist,
            read_events_whitelist,
            events: Events::default(),
            event_deduper: EventDeduper::default(),
            anonymization_config: anonymization_config.into(),
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
        if !self.event_deduper.try_push(event.idempotency_key, now) {
            return;
        }

        if self.anonymization_config.users {
            if let Some(user) = event
                .user
                .as_mut()
                .filter(|u| !self.anonymization_config.exclusions.contains(*u))
            {
                *user = self.anonymize(user);
            }
        }

        if self.anonymization_config.sources {
            if let Some(source) = event
                .source
                .as_mut()
                .filter(|s| !self.anonymization_config.exclusions.contains(*s))
            {
                *source = self.anonymize(source);
            }
        }

        self.events.push(event);
    }

    fn anonymize(&self, value: &str) -> String {
        // Generates a 32 character string from the input value + the salt
        let mut hasher = sha2::Sha256::new();
        hasher.update(value.as_bytes());
        hasher.update(self.salt.get());
        let hash: [u8; 32] = hasher.finalize().into();

        let mut string = String::with_capacity(32);
        for byte in &hash[0..16] {
            write!(string, "{byte:02x}").unwrap();
        }
        string
    }
}

#[derive(Serialize, Deserialize, Default)]
struct AnonymizationConfig {
    users: bool,
    sources: bool,
    exclusions: HashSet<String>,
}

impl From<AnonymizationInitConfig> for AnonymizationConfig {
    fn from(value: AnonymizationInitConfig) -> Self {
        AnonymizationConfig {
            users: value.users.unwrap_or_default(),
            sources: value.sources.unwrap_or_default(),
            exclusions: value.exclusions.unwrap_or_default().into_iter().collect(),
        }
    }
}
