use actix_web::{http, http::StatusCode, HttpResponse};
use serde::Serialize;
use serde_json::json;

pub mod build;

type BoxDynError = Box<dyn std::error::Error + Send + Sync>;

/// general error for returning to the connected client
/// 
/// send back all responses as application/json with a desired status code,
/// error name, message, and optional error source if another error was not
/// handled.
pub struct Error 
{
    status: StatusCode,
    name: String,
    message: String,
    source: Option<BoxDynError>,
    data: Option<serde_json::Value>
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error 
{
    pub fn new() -> Self
    {
        Error {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            name: String::from("InternalError"),
            message: String::from("internal server error"),
            source: None,
            data: None
        }
    }

    pub fn set_status(mut self, status: StatusCode) -> Self
    {
        self.status = status;
        self
    }

    pub fn set_name<N>(mut self, name: N) -> Self
    where
        N: Into<String>
    {
        self.name = name.into();
        self
    }

    pub fn set_message<M>(mut self, message: M) -> Self
    where
        M: Into<String>
    {
        self.message = message.into();
        self
    }

    pub fn set_source<E>(mut self, source: E) -> Self
    where
        E: Into<BoxDynError>
    {
        self.source = Some(source.into());
        self
    }

    pub fn set_data<D>(mut self, data: D) -> Self
    where
        D: Serialize
    {
        self.data = Some(json!(data));
        self
    }
}

impl std::fmt::Debug for Error 
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("status", &self.status)
            .field("name", &self.name)
            .field("message", &self.message)
            .field("source", &self.source)
            .field("data", &self.data)
            .finish()
    }
}

impl std::fmt::Display for Error 
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        write!(f, "Error")
    }
}

impl std::error::Error for Error 
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> 
    {
        self.source.as_ref().map(|e| &**e as _)
    }
}

impl actix_web::error::ResponseError for Error 
{
    fn status_code(&self) -> StatusCode 
    {
        self.status.clone()
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> 
    {
        let mut json = if let Some(data) = self.data.as_ref() 
        {
            json!({"data": data})
        }
        else 
        {
            serde_json::Value::Object(serde_json::Map::<String, serde_json::Value>::new())
        };

        let map = json.as_object_mut().unwrap();
        map.insert("message".into(), serde_json::Value::String(self.message.clone()));
        map.insert("error".into(), serde_json::Value::String(self.name.clone()));

        HttpResponse::build(self.status)
            .insert_header((http::header::CONTENT_TYPE, mime::APPLICATION_JSON))
            .json(json)
    }
}

// From impl

/// helper to implement from traits for various errors
macro_rules! generic_catch {
    ($e:path) => {
        impl From<$e> for Error
        {
            fn from(err: $e) -> Self
            {
                Error::new()
                    .set_source(err)
            }
        }
    };
}

impl From<tokio_postgres::Error> for Error 
{
    fn from(err: tokio_postgres::Error) -> Self 
    {
        Error::new()
            .set_name("DatabaseError")
            .set_message("database error during request")
            .set_source(err)
    }
}

impl From<bb8_postgres::bb8::RunError<tokio_postgres::Error>> for Error 
{
    fn from(error: bb8_postgres::bb8::RunError<tokio_postgres::Error>) -> Self 
    {
        Error::new()
            .set_name("DatabaseError")
            .set_message("database error during request")
            .set_source(error)
    }
}

// std
generic_catch!(std::fmt::Error);
generic_catch!(std::io::Error);

// chrono
generic_catch!(chrono::ParseError);

// lettre
generic_catch!(lettre::error::Error);
generic_catch!(lettre::transport::smtp::Error);

// handlebars
generic_catch!(handlebars::RenderError);

// serde_json
generic_catch!(serde_json::Error);

// uuid
generic_catch!(uuid::Error);

// rand
generic_catch!(rand::Error);

// argon2
generic_catch!(argon2::Error);

// actix_web
// generic_catch!(actix_web::error::Error);
generic_catch!(actix_web::http::header::ToStrError);

// it would be nice if this would work
// impl<E> From<E> for Error
// where
//     E: std::error::Error + Send + Sync
// {
//     fn from(err: E) -> Self 
//     {
//         Error::new()
//             .set_name("InternalError")
//             .set_message("internal server error")
//             .set_source(err)
//     }
// }