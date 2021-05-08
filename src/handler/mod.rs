use actix_web::{web, http, HttpRequest, HttpResponse, Responder, error};
use actix_session::{Session};

use crate::response;
use crate::request::from;
use crate::error as app_error;
use crate::state;

pub mod auth;
pub mod entries;
pub mod mood_fields;
pub mod users;
pub mod account;
pub mod backup;
pub mod admin;

pub async fn handle_get_root(
    session: Session,
    app: web::Data<state::AppState>,
) -> app_error::Result<impl Responder> {
    let conn = &app.get_conn().await?;
    let initiator = from::get_initiator(conn, &session).await?;

    match initiator {
        Some(_) => Ok(response::redirect_to_path("/entries")),
        None => Ok(response::redirect_to_path("/auth/login"))
    }
}

#[allow(dead_code)]
pub async fn okay() -> impl Responder {
    HttpResponse::Ok().body("okay")
}

pub async fn handle_get_data(
    initiator: from::Initiator
) -> app_error::Result<impl Responder> {
    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful",
            initiator.user
        )
    ))
}

pub fn handle_json_error(
    err: error::JsonPayloadError,
    _req: &HttpRequest
) -> error::Error {
    let err_str = err.to_string();
    error::InternalError::from_response(
        err, 
        response::json::respond_json(
            http::StatusCode::CONFLICT,
            response::json::ErrorJSON::build_with_err(
                "given json is not valid",
                "invalid json",
                err_str
            )
        )
    ).into()
}