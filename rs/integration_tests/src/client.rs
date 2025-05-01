use candid::{CandidType, Principal};
use event_store_canister::{EventsArgs, EventsResponse, PushEventsArgs};
use pocket_ic::{PocketIc, RejectResponse};
use serde::de::DeserializeOwned;

pub fn events(
    env: &PocketIc,
    sender: Principal,
    canister_id: Principal,
    args: &EventsArgs,
) -> EventsResponse {
    execute_query(env, sender, canister_id, "events", args)
}

pub fn push_events(
    env: &mut PocketIc,
    sender: Principal,
    canister_id: Principal,
    args: &PushEventsArgs,
) {
    execute_update_no_response(env, sender, canister_id, "push_events", args)
}

fn execute_query<P: CandidType, R: CandidType + DeserializeOwned>(
    env: &PocketIc,
    sender: Principal,
    canister_id: Principal,
    method_name: &str,
    payload: &P,
) -> R {
    unwrap_response(env.query_call(
        canister_id,
        sender,
        method_name,
        candid::encode_one(payload).unwrap(),
    ))
}

#[allow(dead_code)]
fn execute_update<P: CandidType, R: CandidType + DeserializeOwned>(
    env: &mut PocketIc,
    sender: Principal,
    canister_id: Principal,
    method_name: &str,
    payload: &P,
) -> R {
    unwrap_response(env.update_call(
        canister_id,
        sender,
        method_name,
        candid::encode_one(payload).unwrap(),
    ))
}

fn execute_update_no_response<P: CandidType>(
    env: &mut PocketIc,
    sender: Principal,
    canister_id: Principal,
    method_name: &str,
    payload: &P,
) {
    env.update_call(
        canister_id,
        sender,
        method_name,
        candid::encode_one(payload).unwrap(),
    )
    .unwrap();
}

fn unwrap_response<R: CandidType + DeserializeOwned>(
    response: Result<Vec<u8>, RejectResponse>,
) -> R {
    match response {
        Ok(bytes) => candid::decode_one(&bytes).unwrap(),
        Err(error) => panic!("{error}"),
    }
}
