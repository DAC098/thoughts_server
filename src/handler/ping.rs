use actix_web::{http, Responder, HttpResponse};

use crate::net::http::error;

pub async fn handle_get() -> error::Result<impl Responder> {
    Ok(
        HttpResponse::build(http::StatusCode::OK)
            .body("pong")
    )
}