use actix_web::{web, http, Responder};
use actix_session::{Session};

use crate::db::user_sessions;
use crate::response;
use crate::state;
use crate::request::from;

use response::error;

pub async fn handle_post(
    session: Session,
    app: web::Data<state::AppState>,
) -> error::Result<impl Responder> {
    let conn = &mut app.get_conn().await?;
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