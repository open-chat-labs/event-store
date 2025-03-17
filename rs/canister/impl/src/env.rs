use candid::Principal;
use event_store_canister::TimestampMillis;

pub fn time() -> TimestampMillis {
    ic_cdk::api::time() / 1_000_000
}

pub fn caller() -> Principal {
    ic_cdk::api::msg_caller()
}
