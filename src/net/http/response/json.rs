use actix_web::{http, HttpResponse, HttpResponseBuilder};
use actix_web::http::header::TryIntoHeaderPair;
use serde::Serialize;
use serde_json::json;

use super::error;

pub struct JsonBuilder {
    builder: HttpResponseBuilder,
    message: String,
    error: Option<String>,
    reason: Option<String>,
    time: Option<chrono::DateTime<chrono::Utc>>
}

impl JsonBuilder {
    pub fn new(status: http::StatusCode) -> JsonBuilder {
        JsonBuilder {
            builder: HttpResponse::build(status),
            message: "successful".into(),
            error: None,
            reason: None,
            time: None,
        }
    }

    pub fn set_message<M>(mut self, message: M) -> JsonBuilder
    where
        M: Into<String>
    {
        self.message = message.into();
        self
    }

    pub fn set_error<E>(mut self, error: E) -> JsonBuilder
    where
        E: Into<String>
    {
        self.error = Some(error.into());
        self
    }

    pub fn set_reason<R>(mut self, reason: R) -> JsonBuilder
    where
        R: Into<String>
    {
        self.error = Some(reason.into());
        self
    }

    // pub fn set_time(mut self, time: chrono::DateTime<chrono::Utc>) -> JsonBuilder {
    //     self.time = Some(time);
    //     self
    // }

    // pub fn set_time_now(mut self) -> JsonBuilder {
    //     self.time = Some(chrono::Utc::now());
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
        let mut json = if let Some(data) = data {
            json!({"data": data})
        } else {
            serde_json::Value::Object(serde_json::Map::<String, serde_json::Value>::new())
        };

        let map = json.as_object_mut().unwrap();
        map.insert("message".into(), serde_json::Value::String(self.message));

        if let Some(error) = self.error {
            map.insert("error".into(), serde_json::Value::String(error));
        }

        if let Some(reason) = self.reason {
            map.insert("reason".into(), serde_json::Value::String(reason));
        }

        if let Some(time) = self.time {
            map.insert("timestamp".into(), serde_json::Value::Number(serde_json::Number::from(time.timestamp())));
        }

        self.builder.insert_header((http::header::CONTENT_TYPE, "application/json"));
        Ok(self.builder.json(map))
    }

    #[inline]
    pub fn build_empty(self) -> error::Result<HttpResponse> {
        self.build(None::<()>)
    }
}