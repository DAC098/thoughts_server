use actix_web::{http, HttpRequest, Responder, error};
use actix_session::{Session};

use crate::response;
use crate::request::from;
use crate::state;

use response::error as app_error;

pub mod ping;
pub mod auth;
pub mod entries;
pub mod custom_fields;
pub mod users;
pub mod account;
pub mod backup;
pub mod admin;
pub mod tags;
pub mod email;
pub mod global;

pub async fn handle_get(
    session: Session,
    db: state::WebDbState,
) -> app_error::Result<impl Responder> {
    let conn = &*db.get_conn().await?;

    match from::get_initiator(conn, &session).await? {
        Some(_) => Ok(response::redirect_to_path("/entries")),
        None => Ok(response::redirect_to_path("/auth/login"))
    }
}

#[allow(dead_code)]
pub async fn okay() -> impl Responder {
    response::okay_response()
}

pub fn handle_json_error(
    err: error::JsonPayloadError,
    _req: &HttpRequest
) -> error::Error {
    let err_str = err.to_string();

    let response = match &err {
        error::JsonPayloadError::OverflowKnownLength {
            length, limit
        } => {
            response::json::respond_json(
                http::StatusCode::INTERNAL_SERVER_ERROR, 
                response::json::ErrorJSON::build(
                    format!("given json payload is too large. length: {} max size: {}", length, limit),
                    "JsonPayloadTooLarge"
                )
            )
        },
        error::JsonPayloadError::Overflow { limit } => {
            response::json::respond_json(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                response::json::ErrorJSON::build(
                    format!("given json payload is too large. max size: {}", limit),
                    "JsonPayloadTooLarge"
                )
            )
        },
        error::JsonPayloadError::ContentType => {
            response::json::respond_json(
                http::StatusCode::CONFLICT,
                response::json::ErrorJSON::build(
                    "json content type error",
                    "JsonInvalidContentType"
                )
            )
        },
        error::JsonPayloadError::Serialize(err) |
        error::JsonPayloadError::Deserialize(err) => {
            if err.is_io() {
                response::json::respond_json(
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                    response::json::ErrorJSON::build(
                        "json io error",
                        "JsonIOError"
                    )
                )
            } else if err.is_syntax() {
                response::json::respond_json(
                    http::StatusCode::BAD_REQUEST,
                    response::json::ErrorJSON::build(
                        format!("json syntax error. line: {} column: {}", err.line(), err.column()),
                        "JsonSyntaxError"
                    )
                )
            } else if err.is_data() {
                response::json::respond_json(
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                    response::json::ErrorJSON::build(
                        "json data error",
                        "JsonDataError"
                    )
                )
            } else if err.is_eof() {
                response::json::respond_json(
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                    response::json::ErrorJSON::build(
                        "json unexpected end of input",
                        "JsonEof"
                    )
                )
            } else {
                response::json::respond_json(
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                    response::json::ErrorJSON::build(
                        "given json is not valid",
                        "JsonInvalid"
                    )
                )
            }
        },
        _ => response::json::respond_json(
            http::StatusCode::CONFLICT,
            response::json::ErrorJSON::build_with_err(
                "given json is not valid",
                "JsonInvalid",
                err_str
            )
        )
    };

    error::InternalError::from_response(err, response).into()
}

pub async fn handle_not_found(
    req: HttpRequest,
    session: Session,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
) -> app_error::Result<impl Responder> {
    let accept_html = response::check_if_html_req(&req, true)?;
    let conn = &*db.get_conn().await?;
    let initiator_opt = from::get_initiator(conn, &session).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(initiator_opt.unwrap().user))?)
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