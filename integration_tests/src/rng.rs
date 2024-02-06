use candid::Principal;
use rand::RngCore;

pub fn random_principal() -> Principal {
    let random_bytes = rand::thread_rng().next_u32().to_ne_bytes();

    Principal::from_slice(&random_bytes)
}

pub fn random_string() -> String {
    rand::thread_rng().next_u32().to_string()
}

pub fn random_bytes() -> Vec<u8> {
    let mut bytes = [0; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.to_vec()
}
