use actix_web::{http, HttpResponse, HttpResponseBuilder};
use actix_web::http::header::TryIntoHeaderPair;
use serde::Serialize;

use super::error;
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

pub fn respond_json<T>(status: http::StatusCode, data: T) -> HttpResponse
where
    T: Serialize 
{
    let mut builder = HttpResponse::build(status);
    builder.insert_header((http::header::CONTENT_TYPE, "application/json"));
    builder.json(data)
}

pub struct JsonBuilder {
    builder: HttpResponseBuilder,
    message: String,
    error: Option<String>,
    time: Option<chrono::DateTime<chrono::Utc>>
}

impl JsonBuilder {
    pub fn new(status: http::StatusCode) -> JsonBuilder {
        JsonBuilder { 
            builder: HttpResponse::build(status), 
            message: "successful".into(), 
            error: None, 
            time: Some(chrono::Utc::now())
        }
    }

    pub fn set_message<M>(mut self, message: M) -> JsonBuilder
    where
        M: Into<String>
    {
        self.message = message.into();
        self
    }

    pub fn set_error(mut self, error: Option<String>) -> JsonBuilder {
        self.error = error;
        self
    }

    // pub fn set_time(mut self, time: Option<chrono::DateTime<chrono::Utc>>) -> JsonBuilder {
    //     self.time = time;
    //     self
    // }

    pub fn insert_header(mut self, header: impl TryIntoHeaderPair) -> JsonBuilder {
        self.builder.insert_header(header);
        self
    }

    pub fn build<T>(mut self, data: Option<T>) -> error::Result<HttpResponse>
    where
        T: Serialize
    {
        let mut map = serde_json::Map::new();
        map.insert("message".into(), serde_json::Value::String(self.message));

        if let Some(error) = self.error {
            map.insert("error".into(), serde_json::Value::String(error));
        }

        if let Some(time) = self.time {
            map.insert("timestamp".into(), serde_json::Value::Number(serde_json::Number::from(time.timestamp())));
        }

        if let Some(data) = data {
            map.insert("data".into(), serde_json::to_value(data)?);
        }

        self.builder.insert_header((http::header::CONTENT_TYPE, "application/json"));
        Ok(self.builder.json(map))
    }
}