use actix_web::{web, Responder};
use serde::{Deserialize};

use crate::state;
use crate::response;
use crate::util;

use response::error;

#[derive(Deserialize)]
pub struct QueryOptions {
    id: String
}

pub async fn handle_get(
    app_data: web::Data<state::AppState>,
    info: web::Query<QueryOptions>,
) -> error::Result<impl Responder> {
    let app = app_data.into_inner();

    if !app.email.enabled {
        return Ok(response::json::respond_message("email disabled for server"));
    }

    let conn = &mut *app.get_conn().await?;

    let record = conn.query(
        "select owner, issued from email_verifications where key_id = $1",
        &[&info.id]
    ).await?;

    if record.is_empty() {
        return Ok(response::json::respond_message("verification id not found"));
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
        return Ok(response::json::respond_message("verification id has expired"));
    }

    transaction.execute(
        "update users set email_verified = true where id = $1",
        &[&owner]
    ).await?;

    transaction.commit().await?;

    Ok(response::json::respond_message("email verified"))
}