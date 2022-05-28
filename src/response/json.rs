use actix_web::{http, HttpResponse};
use actix_web::http::header::TryIntoHeaderPair;
use serde::{Serialize};

use crate::util;

#[derive(Serialize)]
pub struct ErrorJSON {
    r#type: String,
    message: String,
    date: String,
    error: Option<String>
}

impl ErrorJSON {
    pub fn build<M, T>(m: M, t: T) -> ErrorJSON
    where
        M: Into<String>,
        T: Into<String>
    {
        ErrorJSON {
            r#type: t.into(),
            message: m.into(),
            date: util::time::now_rfc3339(),
            error: None
        }
    }

    pub fn build_with_err<M, T, E>(m: M, t: T, e: E) -> ErrorJSON
    where
        M: Into<String>,
        T: Into<String>,
        E: ToString
    {
        ErrorJSON {
            r#type: t.into(),
            message: m.into(),
            date: util::time::now_rfc3339(),
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
            date: util::time::now_rfc3339(),
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

pub fn respond_json<T>(status: http::StatusCode, data: T) -> HttpResponse
where
    T: Serialize 
{
    let mut builder = HttpResponse::build(status);
    builder.insert_header((http::header::CONTENT_TYPE, "application/json"));
    builder.json(data)
}

#[allow(dead_code)]
pub fn respond_json_headers<T, H>(status: http::StatusCode, data: T, headers: Vec<H>) -> HttpResponse
where
    T: Serialize,
    H: TryIntoHeaderPair
{
    let mut builder = HttpResponse::build(status);

    for header in headers {
        builder.insert_header(header);
    }

    builder.insert_header((http::header::CONTENT_TYPE, "application/json"));
    builder.json(data)
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