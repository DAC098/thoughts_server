use std::path::PathBuf;

use actix_files::NamedFile;
use actix_web::http::Method;
use actix_web::{http, HttpRequest, Responder, error};

use crate::request::initiator_from_request;
use crate::response;
use crate::response::json::JsonBuilder;
use crate::state;

use response::error as response_error;

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
    req: HttpRequest,
    db: state::WebDbState,
) -> response_error::Result<impl Responder> {
    let conn = &*db.get_conn().await?;

    match initiator_from_request(conn, &req).await? {
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

pub async fn handle_file_serving(
    req: HttpRequest,
    file_serving: state::WebFileServingState
) -> response_error::Result<impl Responder> {
    if req.method() != Method::GET {
        return JsonBuilder::new(http::StatusCode::METHOD_NOT_ALLOWED)
            .set_error(Some("MethodNotAllowed".into()))
            .set_message("requested method is not accepted by this resource")
            .build(None::<()>)
    }

    let lookup = req.uri().path();
    let mut to_send: Option<PathBuf> = None;

    if let Some(file_path) = file_serving.files.get(lookup) {
        to_send = Some(file_path.clone())
    } else {
        for (key, path) in file_serving.directories.iter() {
            if let Some(stripped) = lookup.strip_prefix(key.as_str()) {
                let mut sanitize = String::with_capacity(stripped.len());
                let mut first = true;

                for value in stripped.split("/") {
                    if value == ".." || value == "." || value.len() == 0 {
                        return JsonBuilder::new(http::StatusCode::BAD_REQUEST)
                            .set_error(Some("MalformedResourcePath".into()))
                            .set_message("resource path given contains invalid segments. \"..\", \".\", and \"\" are not allowed in the path")
                            .build(None::<()>)
                    }

                    if first {
                        first = false;
                    } else {
                        sanitize.push('/');
                    }

                    sanitize.push_str(value);
                }

                let mut file_path = path.clone();
                file_path.push(sanitize);

                to_send = Some(file_path);
                break;
            }
        }
    }

    if let Some(file_path) = to_send {
        Ok(NamedFile::open_async(file_path)
            .await?
            .into_response(&req))
    } else {
        JsonBuilder::new(http::StatusCode::NOT_FOUND)
            .set_error(Some("NotFound".into()))
            .set_message("the requested resource was not found")
            .build(None::<()>)
    }
}