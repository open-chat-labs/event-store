use candid::Principal;
use rand::distributions::{Distribution, Standard};
use rand::Rng;

pub fn random<T>() -> T
where
    Standard: Distribution<T>,
{
    rand::thread_rng().gen()
}

pub fn random_principal() -> Principal {
    let random_bytes = random::<u32>().to_ne_bytes();

    Principal::from_slice(&random_bytes)
}

pub fn random_string() -> String {
    random::<u32>().to_string()
}

pub fn random_bytes() -> Vec<u8> {
    random::<[u8; 32]>().to_vec()
}
