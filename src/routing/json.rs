//! deals with handling json errors from the actix json helper

use actix_web::{http, HttpRequest, error as actix_error};

use crate::net::http::response::json::JsonBuilder;

/// function handle for actix json helper
pub fn handle_json_error(
    err: actix_error::JsonPayloadError,
    _req: &HttpRequest
) -> actix_error::Error {
    let response = match &err {
        actix_error::JsonPayloadError::OverflowKnownLength {
            length, limit
        } => {
            JsonBuilder::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                .set_message(format!("given json payload is too large. length: {} max size: {}", length, limit))
                .set_error("JsonPayloadTooLarge")
                .build_empty()
        },
        actix_error::JsonPayloadError::Overflow { limit } => {
            JsonBuilder::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                .set_message(format!("given json payload is too large. max size: {}", limit))
                .set_error("JsonPayloadTooLarge")
                .build_empty()
        },
        actix_error::JsonPayloadError::ContentType => {
            JsonBuilder::new(http::StatusCode::CONFLICT)
                .set_message("json content type error")
                .set_error("JsonInvalidContentType")
                .build_empty()
        },
        actix_error::JsonPayloadError::Serialize(err) |
        actix_error::JsonPayloadError::Deserialize(err) => {
            if err.is_io() {
                JsonBuilder::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                    .set_message("json io error")
                    .set_error("JsonIOError")
                    .build_empty()
            } else if err.is_syntax() {
                JsonBuilder::new(http::StatusCode::BAD_REQUEST)
                    .set_message(format!("json syntax error. line: {} column: {}", err.line(), err.column()))
                    .set_error("JsonSyntaxError")
                    .build_empty()
            } else if err.is_data() {
                JsonBuilder::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                    .set_message("json data error")
                    .set_error("JsonDataError")
                    .build_empty()
            } else if err.is_eof() {
                JsonBuilder::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                    .set_message("json unexpected end of input")
                    .set_error("JsonEof")
                    .build_empty()
            } else {
                JsonBuilder::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                    .set_message("given json is not valid")
                    .set_error("JsonInvalid")
                    .build_empty()
            }
        },
        _ => JsonBuilder::new(http::StatusCode::CONFLICT)
            .set_message("given json is not valid")
            .set_error("JsonInvalid")
            .set_reason(err.to_string())
            .build_empty()
    }.unwrap();

    actix_error::InternalError::from_response(err, response).into()
}

