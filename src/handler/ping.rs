use actix_web::{http, Responder, HttpResponse};

use crate::response;

pub async fn handle_get() -> response::error::Result<impl Responder> {
    Ok(
        HttpResponse::build(http::StatusCode::OK)
            .body("pong")
    )
}