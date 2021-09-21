use std::pin::Pin;
use actix_web::{web, dev::Payload, FromRequest, HttpRequest};
use actix_session::{UserSession, Session};
use tokio_postgres::{Client};
use futures::Future;

use tlib::db::{users};

use crate::state;
use crate::response::error;

pub struct Initiator {
    pub user: users::User
}

impl Initiator {

    pub fn into_inner(self) -> users::User {
        self.user
    }
    
}

pub fn get_session_token(session: &Session) -> error::Result<Option<uuid::Uuid>> {
    match session.get::<String>("token")? {
        Some(token) => Ok(Some(uuid::Uuid::parse_str(token.as_str())?)),
        None => Ok(None)
    }
}

pub async fn get_initiator(
    conn: &Client,
    session: &Session,
) -> error::Result<Option<Initiator>> {
    if let Some(uuid_token) = get_session_token(session)? {
        let owner_opt = users::find_from_session_token(conn, uuid_token).await?;

        match owner_opt {
            Some(owner) => Ok(Some(Initiator {user: owner})),
            None => Ok(None)
        }
    } else {
        Ok(None)
    }
}

impl FromRequest for Initiator {
    type Config = ();
    type Error = error::ResponseError;
    type Future = Pin<Box<dyn Future<Output = error::Result<Self>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let app = req.app_data::<web::Data<state::db::DBState>>().unwrap().clone();
        let session = UserSession::get_session(req);

        Box::pin(async move {
            let app = app.into_inner();
            let conn = app.get_conn().await?;

            match get_initiator(&conn, &session).await? {
                Some(initiator) => Ok(initiator),
                None => Err(error::ResponseError::Session)
            }
        })
    }
}