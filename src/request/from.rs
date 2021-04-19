use std::pin::Pin;
use actix_web::{web, dev::Payload, FromRequest, HttpRequest};
use actix_session::{UserSession};
use futures::Future;

use crate::db::users;
use crate::db::user_sessions;
use crate::error;
use crate::state;

pub struct Initiator {
    pub user: users::User
}

impl FromRequest for Initiator {
    type Config = ();
    type Error = error::ResponseError;
    type Future = Pin<Box<dyn Future<Output = error::Result<Self>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let app = req.app_data::<web::Data<state::AppState>>().unwrap().clone();
        let session = UserSession::get_session(req);
        let token_result = session.get::<String>("token").map_err(
            |e| error::ResponseError::ActixError(e)
        );

        Box::pin(async move {
            if token_result.is_err() {
                return Err(token_result.err().unwrap());
            }

            if let Some(token) = token_result.unwrap() {
                let conn = &app.get_conn().await?;
                let uuid_token = uuid::Uuid::parse_str(token.as_str()).map_err(
                    |e| error::ResponseError::UuidError(e)
                )?;
                let owner_opt = user_sessions::UserSession::find_token_user(conn, uuid_token).await.map_err(
                    |e| error::ResponseError::PostgresError(e)
                )?;

                if let Some(owner) = owner_opt {
                    return Ok(Initiator { user: owner });
                }
            }

            Err(error::ResponseError::Session)
        })
    }
}