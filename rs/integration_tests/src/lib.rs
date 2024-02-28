#![cfg(test)]
use crate::rng::{random, random_bytes, random_principal, random_string};
use crate::setup::setup_new_env;
use candid::Principal;
use event_store_canister::{EventsArgs, IdempotentEvent, InitArgs, PushEventsArgs};
use pocket_ic::PocketIc;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

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

fn install_canister(init_args: Option<InitArgs>) -> TestEnv {
    let env = setup_new_env();
    let controller = random_principal();
    let wasm = canister_wasm();
    let init_args = init_args.unwrap_or_else(|| InitArgs {
        push_events_whitelist: vec![random_principal()],
        read_events_whitelist: vec![random_principal()],
        anonymization_config: None,
    });

    let canister_id = env.create_canister_with_settings(Some(controller), None);
    env.add_cycles(canister_id, 1_000_000_000_000);
    env.install_canister(
        canister_id,
        wasm,
        candid::encode_one(&init_args).unwrap(),
        Some(controller),
    );

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
    .join("target")
    .join("wasm32-unknown-unknown")
    .join("release")
    .join("event_store_canister_impl.wasm")
}
