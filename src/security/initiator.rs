use std::pin::Pin;

use actix_web::{web, http::StatusCode, dev::Payload, FromRequest, HttpRequest};
use tokio_postgres::GenericClient;
use futures::Future;

use crate::db::{self, tables::{user_sessions::UserSession, users}};
use crate::state;
use crate::net::http::error;
use crate::net::http::cookie::CookieMap;
use super::{mac, state::SecurityState};

pub struct Initiator {
    pub user: users::User,
    pub session: UserSession
}

impl Initiator {
    pub fn into_user(self) -> users::User {
        self.user
    }
}

impl FromRequest for Initiator {
    type Error = error::Error;
    type Future = Pin<Box<dyn Future<Output = std::result::Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let db = req.app_data::<web::Data<state::DBState>>().unwrap().clone();
        let security = req.app_data::<web::Data<SecurityState>>().unwrap().clone();
        let cookies = CookieMap::from(req.headers());

        Box::pin(async move {
            let db = db.into_inner();
            let security = security.into_inner();
            let conn = db.get_conn().await?;

            InitiatorLookup::from_cookie_map(&security, &*conn, &cookies)
                .await?
                .try_into()
        })
    }
}

pub enum InitiatorLookup {
    Found(Initiator),
    InvalidFormat,
    InvalidMAC,
    VerifyFailed,
    SessionNotFound(String),
    SessionExpired(UserSession),
    SessionUnverified(UserSession),
    UserNotFound(UserSession),
    CookieNotFound,
}

impl InitiatorLookup {

    pub async fn from_cookie_map(
        security: &SecurityState,
        conn: &impl GenericClient,
        cookies: &CookieMap
    ) -> std::result::Result<InitiatorLookup, db::error::Error> 
    {
        if let Some(value) = cookies.get_value_ref("session_id") {
            let split = value.split_once('.');

            if split.is_none() {
                return Ok(InitiatorLookup::InvalidFormat)
            }

            let (token, mac) = split.unwrap();
            let decoded_mac = match base64::decode_config(mac, base64::URL_SAFE) {
                Ok(d) => d,
                Err(_err) => {
                    return Ok(InitiatorLookup::InvalidMAC)
                }
            };

            match mac::algo_one_off_verify(
                security.get_signing(), 
                security.get_secret(), 
                &token, 
                &decoded_mac
            ) {
                Ok(valid) => {
                    if !valid {
                        return Ok(InitiatorLookup::VerifyFailed)
                    }
                },
                Err(_error) => {
                    return Ok(InitiatorLookup::InvalidMAC)
                }
            }

            if let Some(session_record) =  UserSession::find_from_token(conn, token).await? {
                let now = chrono::Utc::now();

                if session_record.dropped || session_record.expires < now {
                    return Ok(InitiatorLookup::SessionExpired(session_record));
                }

                if !session_record.verified {
                    return Ok(InitiatorLookup::SessionUnverified(session_record));
                }

                if let Some(user_record) = users::find_from_id(conn, &session_record.owner).await? {
                    Ok(InitiatorLookup::Found(Initiator {
                        user: user_record,
                        session: session_record
                    }))
                } else {
                    Ok(InitiatorLookup::UserNotFound(session_record))
                }
            } else {
                Ok(InitiatorLookup::SessionNotFound(token.into()))
            }
        } else {
            Ok(InitiatorLookup::CookieNotFound)
        }
    }

    pub async fn from_request(
        security: &SecurityState,
        conn: &impl GenericClient,
        req: &HttpRequest
    ) -> std::result::Result<Self, db::error::Error>
    {
        let cookies = CookieMap::from(req);

        InitiatorLookup::from_cookie_map(security, conn, &cookies).await
    }

    pub fn is_valid(&self) -> bool {
        match self {
            InitiatorLookup::Found(_) => true,
            _ => false
        }
    }

    #[inline]
    pub fn is_some(&self) -> bool {
        self.is_valid()
    }

    pub fn unwrap(self) -> Initiator {
        match self {
            InitiatorLookup::Found(initiator) => initiator,
            _ => panic!("no initiator available")
        }
    }

    pub fn get_error(self) -> Option<error::Error> {
        match self {
            InitiatorLookup::Found(_initiator) => None,
            InitiatorLookup::InvalidFormat => Some(error::Error::new()
                .set_status(StatusCode::UNAUTHORIZED)
                .set_name("SessionInvalidFormat")
                .set_message("value for session_id is an invalid format")
            ),
            InitiatorLookup::InvalidMAC => Some(error::Error::new()
                .set_status(StatusCode::UNAUTHORIZED)
                .set_name("SessionInvalidMAC")
                .set_message("MAC value for session_id is invalid")
            ),
            InitiatorLookup::VerifyFailed => Some(error::Error::new()
                .set_status(StatusCode::UNAUTHORIZED)
                .set_name("SessionInvalid")
                .set_message("value for session_id invalid")
            ),
            InitiatorLookup::SessionNotFound(_) => Some(error::Error::new()
                .set_status(StatusCode::NOT_FOUND)
                .set_name("SessionNotFound")
                .set_message("requested session was not found")
            ),
            InitiatorLookup::SessionExpired(_) => Some(error::Error::new()
                .set_status(StatusCode::UNAUTHORIZED)
                .set_name("SessionExpired")
                .set_message("session has expired")
            ),
            InitiatorLookup::SessionUnverified(_) => Some(error::Error::new()
                .set_status(StatusCode::UNAUTHORIZED)
                .set_name("SessionUnverified")
                .set_message("session has not been verified by 2fa")
            ),
            InitiatorLookup::UserNotFound(_) => Some(error::Error::new()
                .set_status(StatusCode::NOT_FOUND)
                .set_name("SessionUserNotFound")
                .set_message("session user was not found")
            ),
            InitiatorLookup::CookieNotFound => Some(error::Error::new()
                .set_status(StatusCode::UNAUTHORIZED)
                .set_name("SessionCookieNotFound")
                .set_message("session_id cookie was not found")
            ),
        }
    }

    pub fn try_into(self) -> error::Result<Initiator> {
        match self {
            InitiatorLookup::Found(initiator) => Ok(initiator),
            _ => Err(self.get_error().unwrap()),
        }
    }
}