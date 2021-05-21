use std::{fmt};
use std::convert::{From};

use actix_web::{
    error::ResponseError as ActixResponseError, 
    http::StatusCode, 
    HttpResponse
};

use crate::response;

pub type Result<R> = std::result::Result<R, ResponseError>;

#[derive(Debug)]
pub enum ResponseError {

    Session,
    InvalidPassword,

    PermissionDenied(String),
    Validation(String),

    UsernameNotFound(String),
    UserIDNotFound(i32),
    EntryNotFound(i32),
    TextEntryNotFound(i32),
    MoodEntryNotFound(i32),
    MoodFieldNotFound(i32),
    TagNotFound(i32),

    UsernameExists(String),
    EmailExists(String),
    EntryExists(String),
    MoodFieldExists(String),

    // specific module errors

    RustFMTError(std::fmt::Error),
    RustIOError(std::io::Error),

    SerdeJsonError(serde_json::Error),

    ActixError(actix_web::error::Error),
    HeaderError(actix_web::http::header::ToStrError),
    PostgresError(tokio_postgres::Error),
    BB8Error(bb8_postgres::bb8::RunError<tokio_postgres::Error>),
    Argon2Error(argon2::Error),

    OpensslError(openssl::error::Error),
    OpensslErrorStack(openssl::error::ErrorStack),
    
    UuidError(uuid::Error)
}

impl ResponseError {

    fn error_type(&self) -> &str {
        match &*self {

            ResponseError::Session => "SessionError",
            ResponseError::InvalidPassword => "InvalidPassword",

            ResponseError::PermissionDenied(_) => "PermissionDenied",
            ResponseError::Validation(_) => "ValidationError",

            ResponseError::UsernameNotFound(_) => "UsernameNotFound",
            ResponseError::UserIDNotFound(_) => "UserIDNotFound",
            ResponseError::EntryNotFound(_) => "EntryNotFound",
            ResponseError::TextEntryNotFound(_) => "TextEntryNotFound",
            ResponseError::MoodEntryNotFound(_) => "MoodEntryNotFound",
            ResponseError::MoodFieldNotFound(_) => "MoodFieldNotFound",
            ResponseError::TagNotFound(_) => "TagNotFound",

            ResponseError::UsernameExists(_) => "UsernameExists",
            ResponseError::EmailExists(_) => "EmailExists",
            ResponseError::EntryExists(_) => "EntryExists",
            ResponseError::MoodFieldExists(_) => "MoodFieldExists",

            ResponseError::RustFMTError(_) => "InternalError",
            ResponseError::RustIOError(_) => "InternalError",

            ResponseError::SerdeJsonError(_) => "InternalError",

            ResponseError::ActixError(_) => "InternalError",
            ResponseError::HeaderError(_) => "InternalError",

            ResponseError::PostgresError(_) => "DatabaseError",
            ResponseError::BB8Error(_) => "DatabaseError",
            ResponseError::Argon2Error(_) => "InternalError",

            ResponseError::OpensslError(_) => "InternalError",
            ResponseError::OpensslErrorStack(_) => "InternalError",

            ResponseError::UuidError(_) => "InternalError"
        }
    }

    fn get_msg(&self) -> String {
        match &*self {
        
            ResponseError::Session => "no session found for request".to_owned(),
            ResponseError::InvalidPassword => "invalid password given for account".to_owned(),

            ResponseError::PermissionDenied(s) => s.clone(),
            ResponseError::Validation(s) => s.clone(),

            ResponseError::UsernameNotFound(username) => format!("failed to find the requested username: {}", username),
            ResponseError::UserIDNotFound(id) => format!("failed to find the requested user id: {}", id),
            ResponseError::EntryNotFound(id) => format!("failed to find the requested entry id: {}", id),
            ResponseError::TextEntryNotFound(id) => format!("failed to find the requested text entry id: {}", id),
            ResponseError::MoodEntryNotFound(id) => format!("failed to find the requested mood entry id: {}", id),
            ResponseError::MoodFieldNotFound(id) => format!("failed to find the requested mood field id: {}", id),
            ResponseError::TagNotFound(id) => format!("failed to find the requested tag id: {}", id),

            ResponseError::UsernameExists(_) => format!("given username already exist"),
            ResponseError::EmailExists(_) => format!("given email already exists"),
            ResponseError::EntryExists(created) => format!("given entry date already exists. date: {}", created),
            ResponseError::MoodFieldExists(name) => format!("given mood field already exists. name: {}", name),

            ResponseError::RustFMTError(_) => "internal server error".to_owned(),
            ResponseError::RustIOError(_) => "internal server error".to_owned(),

            ResponseError::SerdeJsonError(_) => "internal server error".to_owned(),

            ResponseError::ActixError(_) => "internal server error".to_owned(),
            ResponseError::HeaderError(_) => "internal server error".to_owned(),

            ResponseError::PostgresError(_) => "database server error".to_owned(),
            ResponseError::BB8Error(_) => "database server error".to_owned(),
            ResponseError::Argon2Error(_) => "internal server error".to_owned(),

            ResponseError::OpensslError(_) => "internal server error".to_owned(),
            ResponseError::OpensslErrorStack(_) => "internal server error".to_owned(),

            ResponseError::UuidError(_) => "internal server error".to_owned()
        }
    }
    
}

impl fmt::Display for ResponseError {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_msg())
    }

}

impl ActixResponseError for ResponseError {

    fn error_response(&self) -> HttpResponse {
        response::json::respond_json(
            self.status_code(),
            response::json::ErrorJSON::build(
                self.get_msg(), 
                self.error_type()
            )
        )
    }
    
    fn status_code(&self) -> StatusCode {
        match &*self {

            ResponseError::Session => StatusCode::UNAUTHORIZED,
            ResponseError::InvalidPassword => StatusCode::UNAUTHORIZED,

            ResponseError::PermissionDenied(_) => StatusCode::UNAUTHORIZED,
            ResponseError::Validation(_) => StatusCode::BAD_REQUEST,

            ResponseError::UsernameNotFound(_) => StatusCode::NOT_FOUND,
            ResponseError::UserIDNotFound(_) => StatusCode::NOT_FOUND,
            ResponseError::EntryNotFound(_) => StatusCode::NOT_FOUND,
            ResponseError::TextEntryNotFound(_) => StatusCode::NOT_FOUND,
            ResponseError::MoodEntryNotFound(_) => StatusCode::NOT_FOUND,
            ResponseError::MoodFieldNotFound(_) => StatusCode::NOT_FOUND,
            ResponseError::TagNotFound(_) => StatusCode::NOT_FOUND,

            ResponseError::UsernameExists(_) => StatusCode::BAD_REQUEST,
            ResponseError::EmailExists(_) => StatusCode::BAD_REQUEST,
            ResponseError::EntryExists(_) => StatusCode::BAD_REQUEST,
            ResponseError::MoodFieldExists(_) => StatusCode::BAD_REQUEST,

            ResponseError::RustFMTError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ResponseError::RustIOError(_) => StatusCode::INTERNAL_SERVER_ERROR,

            ResponseError::SerdeJsonError(_) => StatusCode::INTERNAL_SERVER_ERROR,

            ResponseError::ActixError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ResponseError::HeaderError(_) => StatusCode::INTERNAL_SERVER_ERROR,

            ResponseError::PostgresError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ResponseError::BB8Error(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ResponseError::Argon2Error(_) => StatusCode::INTERNAL_SERVER_ERROR,

            ResponseError::OpensslError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ResponseError::OpensslErrorStack(_) => StatusCode::INTERNAL_SERVER_ERROR,

            ResponseError::UuidError(_) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

// From implementations

impl From<actix_web::error::Error> for ResponseError {

    fn from(error: actix_web::error::Error) -> Self {
        ResponseError::ActixError(error)
    }
    
}

impl From<actix_web::http::header::ToStrError> for ResponseError {

    fn from(error: actix_web::http::header::ToStrError) -> Self {
        ResponseError::HeaderError(error)
    }
    
}

impl From<tokio_postgres::Error> for ResponseError {

    fn from(error: tokio_postgres::Error) -> Self {
        ResponseError::PostgresError(error)
    }
    
}

impl From<bb8_postgres::bb8::RunError<tokio_postgres::Error>> for ResponseError {

    fn from(error: bb8_postgres::bb8::RunError<tokio_postgres::Error>) -> Self {
        ResponseError::BB8Error(error)
    }
    
}

impl From<argon2::Error> for ResponseError {

    fn from(error: argon2::Error) -> Self {
        ResponseError::Argon2Error(error)
    }
    
}

impl From<openssl::error::Error> for ResponseError {

    fn from(error: openssl::error::Error) -> Self {
        ResponseError::OpensslError(error)
    }
    
}

impl From<openssl::error::ErrorStack> for ResponseError {
    
    fn from(error: openssl::error::ErrorStack) -> Self {
        ResponseError::OpensslErrorStack(error)
    }
    
}

impl From<uuid::Error> for ResponseError {

    fn from(error: uuid::Error) -> Self {
        ResponseError::UuidError(error)
    }
    
}

impl From<std::fmt::Error> for ResponseError {

    fn from(error: std::fmt::Error) -> Self {
        ResponseError::RustFMTError(error)
    }
    
}

impl From<std::io::Error> for ResponseError {

    fn from(error: std::io::Error) -> Self {
        ResponseError::RustIOError(error)
    }
    
}

impl From<serde_json::Error> for ResponseError {
    
    fn from(error: serde_json::Error) -> Self {
        ResponseError::SerdeJsonError(error)
    }
    
}