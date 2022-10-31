use std::pin::Pin;

use actix_web::{web, dev::Payload, FromRequest, HttpRequest};
use tokio_postgres::GenericClient;
use futures::Future;

use crate::db::user_sessions::UserSession;
use crate::db::users;
use crate::state;
use crate::net::http::error;
use crate::net::http::cookie::CookieMap;
use super::mac;

pub struct Initiator {
    pub user: users::User,
    pub session: UserSession
}

impl Initiator {
    pub fn into_user(self) -> users::User {
        self.user
    }

    // pub fn into_session(self) -> UserSession {
    //     self.session
    // }
}

pub async fn initiator_from_cookie_map(security: &state::SecurityState, conn: &impl GenericClient, cookies: &CookieMap) -> error::Result<Option<Initiator>> {
    if let Some(value) = cookies.get_value_ref("session_id") {
        let split = value.split_once('.');

        if split.is_none() {
            log::debug!(
                "invalid session cookie format. cookie [{:?}]",
                value
            );

            return Err(error::ResponseError::PermissionDenied(
                "invalid session cookie format".into()
            ))
        }

        let (token, mac) = split.unwrap();
        let decoded_mac = match base64::decode_config(mac, base64::URL_SAFE) {
            Ok(d) => d,
            Err(_err) => {
                log::debug!(
                    "failed to parse base64 mac. mac [{:?}]",
                    mac
                );

                return Err(error::ResponseError::PermissionDenied(
                    "invalid session cookie".into()
                ))
            }
        };

        match mac::one_off_verify(security.get_secret().as_bytes(), token.as_bytes(), &decoded_mac) {
            mac::VerifyResult::Valid => {}, // all good
            mac::VerifyResult::Invalid => {
                log::debug!(
                    "session cookie failed validation. cookie [{:?}] mac [{:?}]",
                    token,
                    decoded_mac
                );

                return Err(error::ResponseError::PermissionDenied(
                    "invalid session cookie".into()
                ))
            },
            mac::VerifyResult::InvalidLength => {
                log::debug!(
                    "given mac length is too long. mac [{:?}]",
                    decoded_mac
                );

                return Err(error::ResponseError::PermissionDenied(
                    "invalid session cookie".into()
                ))
            }
        }

        if let Some(session_record) =  UserSession::find_from_token(conn, token).await? {
            let now = chrono::Utc::now();

            if session_record.dropped || session_record.expires < now {
                return Ok(None);
            }

            if let Some(user_record) = users::find_from_id(conn, &session_record.owner).await? {
                Ok(Some(Initiator {
                    user: user_record,
                    session: session_record
                }))
            } else {
                Err(error::ResponseError::UserIDNotFound(session_record.owner))
            }
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

pub async fn initiator_from_request(security: &state::SecurityState, conn: &impl GenericClient, req: &HttpRequest) -> error::Result<Option<Initiator>> {
    let cookies = CookieMap::from(req);

    initiator_from_cookie_map(security, conn, &cookies).await
}

impl FromRequest for Initiator {
    type Error = error::ResponseError;
    type Future = Pin<Box<dyn Future<Output = error::Result<Self>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let db = req.app_data::<web::Data<state::DBState>>().unwrap().clone();
        let security = req.app_data::<web::Data<state::SecurityState>>().unwrap().clone();
        let cookies = CookieMap::from(req.headers());

        Box::pin(async move {
            let db = db.into_inner();
            let security = security.into_inner();
            let conn = db.get_conn().await?;

            match initiator_from_cookie_map(&security, &*conn, &cookies).await? {
                Some(initiator) => Ok(initiator),
                None => Err(error::ResponseError::Session)
            }
        })
    }
}