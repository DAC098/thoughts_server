use std::pin::Pin;
use actix_web::{web, dev::Payload, FromRequest, HttpRequest};
use actix_session::{UserSession, Session};
use tokio_postgres::{Client};
use futures::Future;

use crate::db::users;
use crate::db::user_sessions;
use crate::error;
use crate::state;

pub struct Initiator {
    pub user: users::User
}

pub async fn get_initiator(
    conn: &Client,
    session: Session,
) -> error::Result<Option<Initiator>> {
    if let Some(token) = session.get::<String>("token")? {
        let uuid_token = uuid::Uuid::parse_str(token.as_str())?;
        let owner_opt = user_sessions::find_token_user(conn, uuid_token).await?;

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
        let app_data = req.app_data::<web::Data<state::AppState>>().unwrap().clone();
        let session = UserSession::get_session(req);

        Box::pin(async move {
            let app = app_data.into_inner();
            let conn = app.pool.get().await?;

            match get_initiator(&conn, session).await? {
                Some(initiator) => Ok(initiator),
                None => Err(error::ResponseError::Session)
            }
        })
    }
}