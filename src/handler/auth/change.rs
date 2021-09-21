use actix_web::{web, http, Responder};
use serde::{Deserialize};

use crate::response;
use crate::state;
use crate::request::from;
use crate::security;

use response::error;

#[derive(Deserialize)]
pub struct ChangePasswordJson {
    current_password: String,
    new_password: String
}

pub async fn handle_post(
    initiator: from::Initiator,
    db: state::WebDbState,
    posted: web::Json<ChangePasswordJson>,
) -> error::Result<impl Responder> {
    let conn = &mut *db.get_conn().await?;
    let result = conn.query_one(
        "select id, hash from users where id = $1",
        &[&initiator.user.id]
    ).await?;

    security::verify_password(result.get(1), &posted.current_password)?;

    let hash = security::generate_new_hash(&posted.new_password)?;
    let transaction = conn.transaction().await?;
    let _insert_result = transaction.execute(
        "update users set hash = $1 where id = $2",
        &[&hash, &initiator.user.id]
    ).await?;

    transaction.commit().await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::<Option<()>>::build("password changed", None)
    ))
}