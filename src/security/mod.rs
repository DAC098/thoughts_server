use argon2::{Config, ThreadMode, Variant, Version};

use crate::error;

pub mod assert;

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
    let mut salt: [u8; 64] = [0; 64];
    openssl::rand::rand_bytes(&mut salt)?;

    Ok(argon2::hash_encoded(
        &password.as_bytes(), 
        &salt,
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
        Err(error::ResponseError::InvalidPassword)
    } else {
        Ok(())
    }
}
