//! handles changing passwords

use actix_web::{web, http, Responder};
use serde::Deserialize;

use crate::net::http::error;
use crate::net::http::response::json::JsonBuilder;
use crate::state;
use crate::security::Initiator;
use crate::security;

#[derive(Deserialize)]
pub struct ChangePasswordJson {
    current_password: String,
    new_password: String
}

/// updates password for currently authorized user
///
/// POST /auth/change
pub async fn handle_post(
    initiator: Initiator,
    db: state::WebDbState,
    posted: web::Json<ChangePasswordJson>,
) -> error::Result<impl Responder> {
    let conn = &mut *db.get_conn().await?;
    let result = conn.query_one(
        "select id, hash from users where id = $1",
        &[&initiator.user.id]
    ).await?;

    if !security::verify_password(result.get(1), &posted.current_password)? {
        return Err(error::build::invalid_password());
    }

    let hash = security::generate_new_hash(&posted.new_password)?;
    let transaction = conn.transaction().await?;
    let _insert_result = transaction.execute(
        "update users set hash = $1 where id = $2",
        &[&hash, &initiator.user.id]
    ).await?;

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .set_message("password changed")
        .build(None::<()>)
}
