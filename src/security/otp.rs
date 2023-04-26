//! methods, processes, and defaults for using hotp and totp

use std::convert::{TryInto, TryFrom};

pub use rust_otp::{Algo, totp, verify_totp_code, TotpSettings, VerifyResult};

use crate::{db::tables::auth_otp, util};

impl From<auth_otp::Algo> for Algo {
    fn from(algo: auth_otp::Algo) -> Self {
        match algo {
            auth_otp::Algo::SHA1 => Algo::SHA1,
            auth_otp::Algo::SHA256 => Algo::SHA256,
            auth_otp::Algo::SHA512 => Algo::SHA512
        }
    }
}

impl From<&auth_otp::Algo> for Algo {
    fn from(algo: &auth_otp::Algo) -> Self {
        match algo {
            auth_otp::Algo::SHA1 => Algo::SHA1,
            auth_otp::Algo::SHA256 => Algo::SHA256,
            auth_otp::Algo::SHA512 => Algo::SHA512
        }
    }
}

/// potential errors when converting to TotpSettings
pub enum FromError {
    FromIntError
}

impl TryFrom<auth_otp::AuthOtp> for TotpSettings {
    type Error = FromError;

    fn try_from(value: auth_otp::AuthOtp) -> Result<Self, Self::Error> {
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
            window_after: 1,
            now: None
        })
    }
}

impl TryFrom<&auth_otp::AuthOtp> for TotpSettings {
    type Error = FromError;

    fn try_from(value: &auth_otp::AuthOtp) -> Result<Self, Self::Error> {
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
            window_after: 1,
            now: None
        })
    }
}
