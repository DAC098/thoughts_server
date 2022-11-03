use actix_web::http::StatusCode;
use argon2::{Config, ThreadMode, Variant, Version};
use rand::RngCore;

use crate::net::http::error;

pub mod mac;

pub mod initiator;
pub use initiator::*;

pub mod assert;
pub mod permissions;

pub fn get_rand_bytes(size: usize) -> error::Result<Vec<u8>> {
    let mut rand_bytes = vec![0; size];
    rand::thread_rng().try_fill_bytes(rand_bytes.as_mut_slice())?;
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

pub fn verify_password(hash: &str, password: &String) -> error::Result<()> {
    let matches = argon2::verify_encoded(hash, password.as_bytes())?;

    if !matches {
        Err(error::Error::new()
            .set_status(StatusCode::UNAUTHORIZED)
            .set_name("InvalidPassword")
            .set_message("invalid password given for account")
        )
    } else {
        Ok(())
    }
}
