use std::pin::Pin;

use actix_web::{web, http::StatusCode, dev::Payload, FromRequest, HttpRequest};
use tokio_postgres::GenericClient;
use futures::Future;

use crate::db;
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
}

pub enum InitiatorLookup {
    Found(Initiator),
    InvalidFormat,
    InvalidMAC,
    VerifyFailed,
    SessionNotFound(String),
    SessionExpired,
    UserNotFound(i32),
    CookieNotFound,
}

impl InitiatorLookup {
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

    pub fn try_into(self) -> std::result::Result<Initiator, error::Error> {
        match self {
            InitiatorLookup::Found(initiator) => Ok(initiator),
            InitiatorLookup::InvalidFormat => Err(error::Error::new()
                .set_status(StatusCode::UNAUTHORIZED)
                .set_name("SessionInvalidFormat")
                .set_message("value for session_id is an invalid format")
            ),
            InitiatorLookup::InvalidMAC => Err(error::Error::new()
                .set_status(StatusCode::UNAUTHORIZED)
                .set_name("SessionInvalidMAC")
                .set_message("MAC value for session_id is invalid")
            ),
            InitiatorLookup::VerifyFailed => Err(error::Error::new()
                .set_status(StatusCode::UNAUTHORIZED)
                .set_name("SessionInvalid")
                .set_message("value for session_id invalid")
            ),
            InitiatorLookup::SessionNotFound(_) => Err(error::Error::new()
                .set_status(StatusCode::NOT_FOUND)
                .set_name("SessionNotFound")
                .set_message("requested session was not found")
            ),
            InitiatorLookup::SessionExpired => Err(error::Error::new()
                .set_status(StatusCode::UNAUTHORIZED)
                .set_name("SessionExpired")
                .set_message("session has expired")
            ),
            InitiatorLookup::UserNotFound(_) => Err(error::Error::new()
                .set_status(StatusCode::NOT_FOUND)
                .set_name("SessionUserNotFound")
                .set_message("session user was not found")
            ),
            InitiatorLookup::CookieNotFound => Err(error::Error::new()
                .set_status(StatusCode::UNAUTHORIZED)
                .set_name("SessionCookieNotFound")
                .set_message("session_id cookie was not found")
            ),
        }
    }
}

pub async fn from_cookie_map(
    security: &state::SecurityState,
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

        match mac::one_off_verify(security.get_secret().as_bytes(), token.as_bytes(), &decoded_mac) {
            mac::VerifyResult::Valid => {}, // all good
            mac::VerifyResult::Invalid => {
                return Ok(InitiatorLookup::VerifyFailed)
            },
            mac::VerifyResult::InvalidLength => {
                return Ok(InitiatorLookup::InvalidMAC)
            }
        }

        if let Some(session_record) =  UserSession::find_from_token(conn, token).await? {
            let now = chrono::Utc::now();

            if session_record.dropped || session_record.expires < now {
                return Ok(InitiatorLookup::SessionExpired);
            }

            if let Some(user_record) = users::find_from_id(conn, &session_record.owner).await? {
                Ok(InitiatorLookup::Found(Initiator {
                    user: user_record,
                    session: session_record
                }))
            } else {
                Ok(InitiatorLookup::UserNotFound(session_record.owner))
            }
        } else {
            Ok(InitiatorLookup::SessionNotFound(token.into()))
        }
    } else {
        Ok(InitiatorLookup::CookieNotFound)
    }
}

pub async fn from_request(
    security: &state::SecurityState,
    conn: &impl GenericClient,
    req: &HttpRequest
) -> std::result::Result<InitiatorLookup, db::error::Error>
{
    let cookies = CookieMap::from(req);

    from_cookie_map(security, conn, &cookies).await
}

impl FromRequest for Initiator {
    type Error = error::Error;
    type Future = Pin<Box<dyn Future<Output = std::result::Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let db = req.app_data::<web::Data<state::DBState>>().unwrap().clone();
        let security = req.app_data::<web::Data<state::SecurityState>>().unwrap().clone();
        let cookies = CookieMap::from(req.headers());

        Box::pin(async move {
            let db = db.into_inner();
            let security = security.into_inner();
            let conn = db.get_conn().await?;

            from_cookie_map(&security, &*conn, &cookies)
                .await?
                .try_into()
        })
    }
}