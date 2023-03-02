//! handles verify email requests

use actix_web::http;
use actix_web::{web, Responder};
use serde::Deserialize;

use crate::state;
use crate::net::http::error;
use crate::net::http::response::json::JsonBuilder;
use crate::util;

#[derive(Deserialize)]
pub struct QueryOptions {
    id: String
}

/// handles a verify email token
/// 
/// GET /auth/verify_email
///
/// currently setup to handle a verify email token via get requests.
pub async fn handle_get(
    db: state::WebDbState,
    email: state::WebEmailState,
    info: web::Query<QueryOptions>,
) -> error::Result<impl Responder> {
    if !email.is_enabled() {
        return JsonBuilder::new(http::StatusCode::OK)
            .set_message("email disabled for server")
            .build(None::<()>);
    }

    let conn = &mut *db.get_conn().await?;

    let record = conn.query(
        "select owner, issued from email_verifications where key_id = $1",
        &[&info.id]
    ).await?;

    if record.is_empty() {
        return JsonBuilder::new(http::StatusCode::OK)
            .set_message("verification id not found")
            .build(None::<()>);
    }

    let owner: i32 = record[0].get(0);
    let now = util::time::now();
    let issued: chrono::DateTime<chrono::Local> = record[0].get(1);

    let transaction = conn.transaction().await?;

    transaction.execute(
        "delete from email_verifications where owner = $1",
        &[&owner]
    ).await?;

    if issued + chrono::Duration::days(3) < now {
        return JsonBuilder::new(http::StatusCode::OK)
            .set_message("verification id has expired")
            .build(None::<()>);
    }

    transaction.execute(
        "update users set email_verified = true where id = $1",
        &[&owner]
    ).await?;

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .set_message("email verified")
        .build(None::<()>)
}
