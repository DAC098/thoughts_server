use actix_web::{http, Responder};
use actix_session::{Session};

use tlib::db::{user_sessions};

use crate::response;
use crate::state;
use crate::request::from;

use response::error;

pub async fn handle_post(
    session: Session,
    db: state::WebDbState,
) -> error::Result<impl Responder> {
    let conn = &mut *db.get_conn().await?;
    let token_opt = from::get_session_token(&session)?;
    session.purge();
    
    if let Some(token) = token_opt {
        let transaction = conn.transaction().await?;
        user_sessions::delete(&transaction, token).await?;

        transaction.commit().await?;
    }

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::<Option<()>>::build(
            "logout successful",
            None
        )
    ))
}