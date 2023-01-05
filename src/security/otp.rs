//! methods, processes, and defaults for using hotp and totp

use std::convert::{TryInto, TryFrom};

use super::mac;
use crate::{db, util};

/// default step for totp
pub const _DEFAULT_STEP: u64 = 30;
/// default digit legnth for totp
pub const _DEFAULT_DIGITS: u32 = 8;

/// available hashs for totp and hotp
pub enum Algo {
    SHA1,
    SHA256,
    SHA512
}

impl From<db::auth_otp::Algo> for Algo {
    fn from(algo: db::auth_otp::Algo) -> Self {
        match algo {
            db::auth_otp::Algo::SHA1 => Algo::SHA1,
            db::auth_otp::Algo::SHA256 => Algo::SHA256,
            db::auth_otp::Algo::SHA512 => Algo::SHA512
        }
    }
}

impl From<&db::auth_otp::Algo> for Algo {
    fn from(algo: &db::auth_otp::Algo) -> Self {
        match algo {
            db::auth_otp::Algo::SHA1 => Algo::SHA1,
            db::auth_otp::Algo::SHA256 => Algo::SHA256,
            db::auth_otp::Algo::SHA512 => Algo::SHA512
        }
    }
}

/// wrapper around mac hashing algorithms
#[inline]
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

/// create an totp hash
pub fn totp<S>(algorithm: &Algo, secret: S, digits: u32, step: u64, time: u64) -> String
where
    S: AsRef<[u8]>
{
    let data = (time / step).to_be_bytes();

    generate_integer_string(algorithm, secret.as_ref(), digits, &data)
}

/// result from totp verification
pub enum VerifyResult {
    Valid,
    Invalid,
    /// code contains a non acii digit
    InvalidCharacters,
    /// code is not the required length
    InvalidLength,
    /// potential issues when getting unix epoch integers
    UnixEpochError,
}

/// settings for totp verification
pub struct TotpSettings {
    pub algo: Algo,
    pub secret: Vec<u8>,
    pub digits: u32,
    pub step: u64,
    pub window_before: u8,
    pub window_after: u8
}

/// potential errors when converting to TotpSettings
pub enum FromError {
    FromIntError
}

impl TryFrom<db::auth_otp::AuthOtp> for TotpSettings {
    type Error = FromError;

    fn try_from(value: db::auth_otp::AuthOtp) -> Result<Self, Self::Error> {
        let Ok(digits) = TryInto::try_into(value.digits) else {
            return Err(FromError::FromIntError);
        };
        let Ok(step) = TryInto::try_into(value.step) else {
            return Err(FromError::FromIntError);
        };

        Ok(Self {
            algo: value.algo.into(),
            secret: value.secret,
            digits,
            step,
            window_before: 1,
            window_after: 1
        })
    }
}

impl TryFrom<&db::auth_otp::AuthOtp> for TotpSettings {
    type Error = FromError;

    fn try_from(value: &db::auth_otp::AuthOtp) -> Result<Self, Self::Error> {
        let Ok(digits) = TryInto::try_into(value.digits.clone()) else {
            return Err(FromError::FromIntError);
        };
        let Ok(step) = TryInto::try_into(value.step.clone()) else {
            return Err(FromError::FromIntError);
        };

        Ok(Self {
            algo: From::from(&value.algo),
            secret: value.secret.clone(),
            digits,
            step,
            window_before: 1,
            window_after: 1
        })
    }
}

/// verify totp code from given settings
/// 
/// checks to make sure that the code contains only ascii digits and that the
/// length is equal to the specified digits. after that the current timestamp
/// is checked first, then window before, then window after. if an overflow
/// happens when creating the window timpestamps a UnixEpocError is returned.
pub fn verify_totp_code(settings: &TotpSettings, code: String) -> VerifyResult {
    let mut len: u32 = 0;

    for ch in code.chars() {
        if !ch.is_ascii_digit() {
            return VerifyResult::InvalidCharacters;
        }

        len += 1;
    }

    if len != settings.digits {
        return VerifyResult::InvalidLength;
    }

    let Some(now) = util::time::unix_epoch_sec_now() else {
        // probably the system date is wrong
        return VerifyResult::UnixEpochError;
    };

    // check now first
    if totp(&settings.algo, &settings.secret, settings.digits, settings.step, now) == code {
        return VerifyResult::Valid;
    }

    // check before now
    for win in 1..=settings.window_before {
        let value = settings.step * (win as u64);
        let Some(time) = now.checked_sub(value) else {
            return VerifyResult::UnixEpochError;
        };

        if totp(&settings.algo, &settings.secret, settings.digits, settings.step, time) == code {
            return VerifyResult::Valid;
        }
    }

    // check after now
    for win in 1..=settings.window_after {
        let value = settings.step * (win as u64);
        let Some(time) = now.checked_add(value) else {
            return VerifyResult::UnixEpochError;
        };

        if totp(&settings.algo, &settings.secret, settings.digits, settings.step, time) == code {
            return VerifyResult::Valid;
        }
    }

    VerifyResult::Invalid
}