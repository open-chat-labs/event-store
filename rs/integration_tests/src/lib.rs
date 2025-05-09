#![cfg(test)]
use crate::rng::{random, random_bytes, random_principal, random_string};
use crate::setup::setup_new_env;
use candid::Principal;
use event_store_canister::{EventsArgs, InitArgs, PushEventsArgs};
use event_store_types::{Anonymizable, IdempotentEvent, Milliseconds};
use pocket_ic::PocketIc;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use test_case::test_case;

mod client;
mod rng;
mod setup;

pub struct TestEnv {
    pub env: PocketIc,
    pub canister_id: Principal,
    pub controller: Principal,
    pub push_principals: Vec<Principal>,
    pub read_principals: Vec<Principal>,
}

#[test]
fn push_then_read_events_succeeds() {
    let TestEnv {
        mut env,
        canister_id,
        push_principals,
        read_principals,
        ..
    } = install_canister(None);

    client::push_events(
        &mut env,
        *push_principals.first().unwrap(),
        canister_id,
        &PushEventsArgs {
            events: (0..10)
                .map(|i| IdempotentEvent {
                    idempotency_key: random(),
                    name: random_string(),
                    timestamp: i,
                    user: None,
                    source: None,
                    payload: random_bytes(),
                })
                .collect(),
        },
    );

    let read_response = client::events(
        &env,
        *read_principals.first().unwrap(),
        canister_id,
        &EventsArgs {
            start: 0,
            length: 5,
        },
    );

    assert_eq!(read_response.events.len(), 5);
    assert_eq!(read_response.events.first().unwrap().index, 0);
    assert_eq!(read_response.events.last().unwrap().index, 4);
    assert_eq!(read_response.latest_event_index, Some(9));
}

#[test_case(true, true)]
#[test_case(false, true)]
#[test_case(true, false)]
#[test_case(false, false)]
fn users_and_source_can_be_anonymized(users: bool, sources: bool) {
    let TestEnv {
        mut env,
        canister_id,
        push_principals,
        read_principals,
        ..
    } = install_canister(Some(InitArgs {
        push_events_whitelist: vec![random_principal()],
        read_events_whitelist: vec![random_principal()],
        time_granularity: None,
    }));

    let user = random_string();
    let source = random_string();

    client::push_events(
        &mut env,
        *push_principals.first().unwrap(),
        canister_id,
        &PushEventsArgs {
            events: vec![IdempotentEvent {
                idempotency_key: random(),
                name: random_string(),
                timestamp: 1000,
                user: Some(Anonymizable::new(user.clone(), users)),
                source: Some(Anonymizable::new(source.clone(), sources)),
                payload: Vec::new(),
            }],
        },
    );

    let event = client::events(
        &env,
        *read_principals.first().unwrap(),
        canister_id,
        &EventsArgs {
            start: 0,
            length: 1,
        },
    )
    .events
    .pop()
    .unwrap();

    let user_returned = event.user.unwrap();
    let source_returned = event.source.unwrap();

    if users {
        assert_eq!(user_returned.len(), 32);
    } else {
        assert_eq!(user_returned, user);
    }

    if sources {
        assert_eq!(source_returned.len(), 32);
    } else {
        assert_eq!(source_returned, source);
    }
}

#[test_case(None)]
#[test_case(Some(100))]
#[test_case(Some(1000))]
fn time_granularity_applied_correctly(time_granularity: Option<Milliseconds>) {
    let TestEnv {
        mut env,
        canister_id,
        push_principals,
        read_principals,
        ..
    } = install_canister(Some(InitArgs {
        push_events_whitelist: vec![random_principal()],
        read_events_whitelist: vec![random_principal()],
        time_granularity,
    }));

    client::push_events(
        &mut env,
        *push_principals.first().unwrap(),
        canister_id,
        &PushEventsArgs {
            events: vec![
                IdempotentEvent {
                    idempotency_key: random(),
                    name: random_string(),
                    timestamp: 1001,
                    user: None,
                    source: None,
                    payload: Vec::new(),
                },
                IdempotentEvent {
                    idempotency_key: random(),
                    name: random_string(),
                    timestamp: 2150,
                    user: None,
                    source: None,
                    payload: Vec::new(),
                },
                IdempotentEvent {
                    idempotency_key: random(),
                    name: random_string(),
                    timestamp: 3299,
                    user: None,
                    source: None,
                    payload: Vec::new(),
                },
            ],
        },
    );

    let events = client::events(
        &env,
        *read_principals.first().unwrap(),
        canister_id,
        &EventsArgs {
            start: 0,
            length: 3,
        },
    )
    .events;

    match time_granularity {
        None => {
            assert_eq!(events[0].timestamp, 1001);
            assert_eq!(events[1].timestamp, 2150);
            assert_eq!(events[2].timestamp, 3299);
        }
        Some(100) => {
            assert_eq!(events[0].timestamp, 1000);
            assert_eq!(events[1].timestamp, 2100);
            assert_eq!(events[2].timestamp, 3200);
        }
        Some(1000) => {
            assert_eq!(events[0].timestamp, 1000);
            assert_eq!(events[1].timestamp, 2000);
            assert_eq!(events[2].timestamp, 3000);
        }
        _ => panic!(),
    }
}

fn install_canister(init_args: Option<InitArgs>) -> TestEnv {
    let env = setup_new_env();
    let controller = random_principal();
    let wasm = canister_wasm();
    let init_args = init_args.unwrap_or_else(|| InitArgs {
        push_events_whitelist: vec![random_principal()],
        read_events_whitelist: vec![random_principal()],
        time_granularity: None,
    });

    let canister_id = env.create_canister_with_settings(Some(controller), None);
    env.add_cycles(canister_id, 1_000_000_000_000);
    env.install_canister(
        canister_id,
        wasm,
        candid::encode_one(&init_args).unwrap(),
        Some(controller),
    );

    // Tick twice to initialize the `salt`
    env.tick();
    env.tick();

    TestEnv {
        env,
        canister_id,
        controller,
        push_principals: init_args.push_events_whitelist,
        read_principals: init_args.read_events_whitelist,
    }
}

fn canister_wasm() -> Vec<u8> {
    let file_path = canister_wasm_path();

    let mut file = File::open(&file_path).unwrap_or_else(|e| {
        panic!(
            "Failed to open file: {}. Error: {e:?}",
            file_path.to_str().unwrap()
        )
    });
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("Failed to read file");
    bytes
}

fn canister_wasm_path() -> PathBuf {
    PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("Failed to read CARGO_MANIFEST_DIR env variable"),
    )
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .join(".dfx")
    .join("ic")
    .join("canisters")
    .join("event_store")
    .join("event_store.wasm.gz")
}
