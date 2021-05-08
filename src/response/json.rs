use actix_web::{http, HttpResponse, dev::HttpResponseBuilder};
use serde::{Serialize};

use crate::time;

#[derive(Serialize)]
pub struct ErrorJSON {
    r#type: String,
    message: String,
    date: String,
    error: Option<String>
}

impl ErrorJSON {
    pub fn build<M: Into<String>>(m: M, t: &str) -> ErrorJSON {
        ErrorJSON {
            r#type: t.to_string(),
            message: m.into(),
            date: time::now_rfc3339(),
            error: None
        }
    }

    pub fn build_with_err<M: Into<String>, E: ToString>(m: M, t: &str, e: E) -> ErrorJSON {
        ErrorJSON {
            r#type: t.to_string(),
            message: m.into(),
            date: time::now_rfc3339(),
            error: Some(e.to_string())
        }
    }
}

#[derive(Serialize)]
pub struct MessageDataJSON<T: Serialize> {
    message: String,
    date: String,
    data: T
}

impl<T: Serialize> MessageDataJSON<T> {

    pub fn build<M: Into<String>>(m: M, d: T) -> MessageDataJSON<T> {
        MessageDataJSON {
            message: m.into(),
            date: time::now_rfc3339(),
            data: d
        }
    }
}

pub fn build_json_response(status: http::StatusCode) -> HttpResponseBuilder {
    let mut builder = HttpResponse::build(status);
    builder.insert_header((http::header::CONTENT_TYPE, "application/json"));

    return builder;
}

pub fn respond_json<T: Serialize>(status: http::StatusCode, data: T) -> HttpResponse {
    build_json_response(status).json(data)
}

pub fn respond_okay() -> HttpResponse {
    respond_json(
        http::StatusCode::OK,
        MessageDataJSON::<Option<()>>::build(
            "okay",
            None
        )
    )
}