use actix_web::{web, http, HttpRequest, Responder, error};
use actix_session::{Session};

use crate::response;
use crate::request::from;
use crate::error as app_error;
use crate::state;

pub mod auth;
pub mod entries;
pub mod custom_fields;
pub mod users;
pub mod account;
pub mod backup;
pub mod admin;
pub mod tags;
pub mod email;

pub async fn handle_get(
    session: Session,
    app: web::Data<state::AppState>,
) -> app_error::Result<impl Responder> {
    let conn = &*app.get_conn().await?;

    match from::get_initiator(conn, &session).await? {
        Some(_) => Ok(response::redirect_to_path("/entries")),
        None => Ok(response::redirect_to_path("/auth/login"))
    }
}

#[allow(dead_code)]
pub async fn okay() -> impl Responder {
    response::okay().await
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

pub async fn handle_not_found(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>,
) -> app_error::Result<impl Responder> {
    log::info!("handle_not_found");
    let accept_html = response::check_if_html_req(&req, true)?;
    let conn = &*app.get_conn().await?;
    let initiator_opt = from::get_initiator(conn, &session).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(Some(initiator_opt.unwrap().user)))
        } else {
            Ok(response::redirect_to_path("/auth/login"))
        }
    } else if initiator_opt.is_none() {
        Err(app_error::ResponseError::Session)
    } else {
        Ok(response::json::respond_json(
            http::StatusCode::NOT_FOUND,
            response::json::only_message("not found")
        ))
    }
}