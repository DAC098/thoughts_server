use std::pin::Pin;

use actix_web::{web, dev::Payload, FromRequest, HttpRequest};
use crate::db::user_sessions::UserSession;
use tokio_postgres::GenericClient;
use futures::Future;

use crate::db::users;
use crate::state;
use crate::response::error;

use super::cookie::CookieMap;

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

pub async fn initiator_from_cookie_map(conn: &impl GenericClient, cookies: &CookieMap) -> error::Result<Option<Initiator>> {
    if let Some(token) = cookies.get_value_ref("session_id") {
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

pub async fn initiator_from_request(conn: &impl GenericClient, req: &HttpRequest) -> error::Result<Option<Initiator>> {
    let cookies = CookieMap::from(req);

    initiator_from_cookie_map(conn, &cookies).await
}

impl FromRequest for Initiator {
    type Error = error::ResponseError;
    type Future = Pin<Box<dyn Future<Output = error::Result<Self>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let app = req.app_data::<web::Data<state::DBState>>().unwrap().clone();
        let cookies = CookieMap::from(req.headers());

        Box::pin(async move {
            let app = app.into_inner();
            let conn = app.get_conn().await?;

            match initiator_from_cookie_map(&*conn, &cookies).await? {
                Some(initiator) => Ok(initiator),
                None => Err(error::ResponseError::Session)
            }
        })
    }
}