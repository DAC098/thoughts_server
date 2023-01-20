use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::db::tables::user_sessions::UserSession;
use crate::net;
use super::{state::SecurityState, get_rand_bytes, mac};

pub const SESSION_ID_BYTES: usize = 48;
pub const SESSION_ID_LEN: usize = 64;

pub struct MemoryStorage {
    data: RwLock<HashMap<String, UserSession>>
}

impl MemoryStorage {
    pub async fn get(&self, token: &String) -> Option<UserSession> {
        let reader = self.data.read().await;

        if let Some(session) = reader.get(token) {
            Some(session.clone())
        } else {
            None
        }
    }

    pub async fn add(&mut self, token: String, session: UserSession) -> () {
        let mut writer = self.data.write().await;

        writer.insert(token, session);
    }

    pub async fn drop(&mut self, token: &String) -> bool {
        let mut writer = self.data.write().await;

        writer.remove(token).is_some()
    }
}

#[inline]
pub fn create_session_id() -> std::result::Result<String, rand::Error> {
    get_rand_bytes(SESSION_ID_BYTES)
        .map(|bytes| base64::encode_config(&bytes, base64::URL_SAFE))
}

/// creates signed string from token
/// 
/// signed version of the provided token with the mac appended to the end
pub fn create_signed(security: &SecurityState, token: &str) -> mac::Result<String> {
    let mac = {
        let bytes = mac::algo_one_off(
            security.get_signing(),
            security.get_secret(),
            &token
        )?;

        base64::encode_config(&bytes, base64::URL_SAFE)
    };

    let mut signed = String::with_capacity(token.len() + mac.len());
    signed.push_str(token);
    signed.push_str(&mac);

    Ok(signed)
}

/// simple error type for retrieve_session_id
pub enum RetrievError {
    InvalidFormat,
    InvalidMac
}

/// returns a valid token from the given value
/// 
/// parses a given string to retrieve the token and verify that it matches
/// the provided mac
pub fn retrieve_session_id<'a>(security: &SecurityState, value: &'a str) -> std::result::Result<Option<&'a str>, RetrievError> {
    let Some(token) = value.get(0..SESSION_ID_LEN) else {
        return Err(RetrievError::InvalidFormat);
    };
    let Some(mac) = value.get(SESSION_ID_LEN..) else {
        return Err(RetrievError::InvalidFormat);
    };

    let Ok(decoded) = base64::decode_config(&mac, base64::URL_SAFE) else {
        return Err(RetrievError::InvalidMac);
    };

    if let Ok(valid) = mac::algo_one_off_verify(
        security.get_signing(),
        security.get_secret(),
        &token, 
        &decoded
    ) {
        if !valid {
            Ok(None)
        } else {
            Ok(Some(token))
        }
    } else {
        Err(RetrievError::InvalidMac)
    }
}

/// generates session_id cookie with given duration and value
/// 
/// - domain information is pulled from the security state session object
/// - the path is set to root
/// - max age is set to the duration specified
/// - same site is strict
/// - http only is set to true
pub fn create_cookie<V>(security: &SecurityState, duration: chrono::Duration, value: V) -> net::http::cookie::SetCookie
where
    V: Into<String>
{
    let mut session_cookie = net::http::cookie::SetCookie::new("session_id", value);
    session_cookie.set_domain(security.get_session().get_domain());
    session_cookie.set_path("/");
    session_cookie.set_max_age(duration);
    session_cookie.set_same_site(net::http::cookie::SameSite::Strict);
    session_cookie.set_http_only(true);

    session_cookie
}