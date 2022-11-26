use std::convert::TryInto;

use super::mac;
use crate::{db::auth_otp::{Algo, AuthOtp}, util};

/// default step for totp
pub const _DEFAULT_STEP: u64 = 30;
/// default digit legnth for totp
pub const _DEFAULT_DIGITS: u32 = 8;

fn one_off(algo: &Algo, secret: &[u8], data: &[u8]) -> mac::Result<Vec<u8>> {
    match algo {
        Algo::SHA1 => mac::one_off_sha1(secret, data),
        Algo::SHA256 => mac::one_off_sha256(secret, data),
        Algo::SHA512 => mac::one_off_sha512(secret, data)
    }
}

/// simple string padding given a string and total digits
/// 
/// this will not truncate the string and will just return if the given string
/// is big enough or is equal to the given digits
fn pad_string(uint_string: String, digits: usize) -> String {
    if uint_string.len() < digits {
        let mut rtn = String::with_capacity(digits);

        for _ in 0..(digits - uint_string.len()) {
            rtn.push('0');
        }

        rtn.push_str(&uint_string);
        rtn
    } else {
        uint_string
    }
}

/// generate integer string for otp algorithms
/// 
/// creates the integer string for the given algorithm. will pad the string
/// if it is not long enough for the given amount of digits.
fn generate_integer_string(algorithm: &Algo, secret: &[u8], digits: u32, data: &[u8]) -> String {
    let hash = one_off(algorithm, secret, data).unwrap();

    // pull in the offset from the last byte in the hash
    let offset = (hash[hash.len() - 1] & 0xf) as usize;
    // since we are only going to be filling 32 bits we can set it as a u32 
    // and not have to worry overflow since a u32 consists of 4 u8's. casting
    // the u8's up should not be an issue
    let binary = 
        ((hash[offset] & 0x7f) as u32) << 24 |
        (hash[offset + 1] as u32) << 16 |
        (hash[offset + 2] as u32) <<  8 |
        (hash[offset + 3] as u32);

    let uint_string = (binary % 10u32.pow(digits)).to_string();
    let digits = digits as usize;

    pad_string(uint_string, digits)
}

/// create an hotp hash
pub fn _hotp<S>(secret: S, digits: u32, counter: u64) -> String
where
    S: AsRef<[u8]>
{
    let counter_bytes = counter.to_be_bytes();
    
    generate_integer_string(&Algo::SHA1, secret.as_ref(), digits, &counter_bytes)
}

// create an totp hash
pub fn totp<S>(algorithm: &Algo, secret: S, digits: u32, step: u64, time: u64) -> String
where
    S: AsRef<[u8]>
{
    let data = (time / step).to_be_bytes();

    generate_integer_string(algorithm, secret.as_ref(), digits, &data)
}

pub enum VerifyResult {
    Valid,
    Invalid,
    InvalidCharacters,
    InvalidLength,
    FromIntError,
    UnixEpochError,
}

pub fn verify_totp_code(otp: &AuthOtp, code: String) -> VerifyResult {
    let Ok(digits) = TryInto::<u32>::try_into(otp.digits) else {
        return VerifyResult::FromIntError;
    };
    let mut len: u32 = 0;

    for ch in code.chars() {
        if !ch.is_ascii_digit() {
            return VerifyResult::InvalidCharacters;
        }

        len += 1;
    }

    if len != digits {
        return VerifyResult::InvalidLength;
    }

    let Ok(step) = TryInto::<u64>::try_into(otp.step) else {
        return VerifyResult::FromIntError;
    };
    let Some(now) = util::time::unix_epoch_sec_now() else {
        return VerifyResult::UnixEpochError;
    };
    let prev = now - step;
    let next = now + step;

    // check now first
    if totp(&otp.algo, &otp.secret, digits, step, now) == code {
        return VerifyResult::Valid;
    }

    // check before now
    if totp(&otp.algo, &otp.secret, digits, step, prev) == code {
        return VerifyResult::Valid;
    }

    // check after now
    if totp(&otp.algo, &otp.secret, digits, step, next) == code {
        return VerifyResult::Valid;
    }

    VerifyResult::Invalid
}