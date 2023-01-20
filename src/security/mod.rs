use argon2::{Config, ThreadMode, Variant, Version};
use rand::RngCore;

use crate::net::http::error;

pub mod mac;
pub mod otp;

pub mod session;

pub mod initiator;
pub use initiator::*;

pub mod assert;
pub mod permissions;

pub mod state;

/// simple get random bytes with a given size
/// 
/// uses [rand::thread_rng] to fill the Vec with the given size
pub fn get_rand_bytes(size: usize) -> std::result::Result<Vec<u8>, rand::Error> {
    let mut rng = rand::thread_rng();
    let mut rand_bytes = vec!(0u8; size);

    rng.try_fill_bytes(rand_bytes.as_mut_slice())?;

    Ok(rand_bytes)
}

pub fn default_argon2_config() -> Config<'static> {
    Config {
        variant: Variant::Argon2i,
        version: Version::Version13,
        mem_cost: 65536,
        time_cost: 10,
        lanes: 4,
        thread_mode: ThreadMode::Parallel,
        secret: &[],
        ad: &[],
        hash_length: 32
    }
}

pub fn generate_new_hash_with_config(
    password: &String, 
    config: &Config
) -> error::Result<String> {
    let salt = get_rand_bytes(64)?;

    Ok(argon2::hash_encoded(
        &password.as_bytes(), 
        salt.as_slice(),
        &config
    )?)
}

pub fn generate_new_hash(password: &String) -> error::Result<String> {
    let config = default_argon2_config();

    generate_new_hash_with_config(
        password, 
        &config
    )
}

pub fn verify_password(hash: &str, password: &String) -> error::Result<bool> {
    Ok(argon2::verify_encoded(hash, password.as_bytes())?)
}