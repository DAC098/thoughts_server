//! health check request

use actix_web::{http, Responder, HttpResponse};

use crate::net::http::error;

/// handles ping request
///
/// GET /ping
///
/// no other information is currently sent back other than a text response
/// of "pong"
pub async fn handle_get() -> error::Result<impl Responder> {
    Ok(
        HttpResponse::build(http::StatusCode::OK)
            .body("pong")
    )
}
