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
    pub fn build<M>(m: M, t: &str) -> ErrorJSON
    where
        M: Into<String>
    {
        ErrorJSON {
            r#type: t.to_string(),
            message: m.into(),
            date: time::now_rfc3339(),
            error: None
        }
    }

    pub fn build_with_err<M, E>(m: M, t: &str, e: E) -> ErrorJSON
    where
        M: Into<String>,
        E: ToString
    {
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

    pub fn build<M>(m: M, d: T) -> MessageDataJSON<T>
    where
        M: Into<String>
    {
        MessageDataJSON {
            message: m.into(),
            date: time::now_rfc3339(),
            data: d
        }
    }
}

pub fn only_message<M>(m: M) -> MessageDataJSON<Option<()>>
where
    M: Into<String> 
{
    MessageDataJSON::build(m, None)
}

pub fn build_json_response(status: http::StatusCode) -> HttpResponseBuilder {
    let mut builder = HttpResponse::build(status);
    builder.insert_header((http::header::CONTENT_TYPE, "application/json"));

    return builder;
}

pub fn respond_json<T>(status: http::StatusCode, data: T) -> HttpResponse
where
    T: Serialize
{
    build_json_response(status).json(data)
}

pub fn respond_okay() -> HttpResponse {
    respond_json(
        http::StatusCode::OK,
        only_message("okay")
    )
}

pub fn respond_message<T>(msg: T) -> HttpResponse 
where
    T: Into<String>
{
    respond_json(
        http::StatusCode::OK,
        only_message(msg)
    )
}