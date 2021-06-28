use std::{fmt};
use std::convert::{From};

use actix_web::{
    error::ResponseError as ActixResponseError, 
    http::StatusCode, 
    HttpResponse
};

use crate::db;
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
    CustomFieldNotFound(i32),
    TagNotFound(i32),
    EntryMarkerNotFound(i32),

    UsernameExists(String),
    EmailExists(String),
    EntryExists(String),
    CustomFieldExists(String),

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
    
    UuidError(uuid::Error),

    EmailSmtpError(lettre::transport::smtp::Error),
    EmailBuilderError(lettre::error::Error)
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
            ResponseError::CustomFieldNotFound(_) => "CustomFieldNotFound",
            ResponseError::TagNotFound(_) => "TagNotFound",
            ResponseError::EntryMarkerNotFound(_) => "EntryMarkerNotFound",

            ResponseError::UsernameExists(_) => "UsernameExists",
            ResponseError::EmailExists(_) => "EmailExists",
            ResponseError::EntryExists(_) => "EntryExists",
            ResponseError::CustomFieldExists(_) => "MoodFieldExists",

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

            ResponseError::UuidError(_) => "InternalError",

            ResponseError::EmailSmtpError(_) => "InternalError",
            ResponseError::EmailBuilderError(_) => "InternalError"
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
            ResponseError::CustomFieldNotFound(id) => format!("failed to find the requested custom field id: {}", id),
            ResponseError::TagNotFound(id) => format!("failed to find the requested tag id: {}", id),
            ResponseError::EntryMarkerNotFound(id) => format!("failed to find the requested marker id: {}", id),

            ResponseError::UsernameExists(_) => format!("given username already exist"),
            ResponseError::EmailExists(_) => format!("given email already exists"),
            ResponseError::EntryExists(created) => format!("given entry date already exists. date: {}", created),
            ResponseError::CustomFieldExists(name) => format!("given mood field already exists. name: {}", name),

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

            ResponseError::UuidError(_) => "internal server error".to_owned(),

            ResponseError::EmailSmtpError(_) => "internal server error".to_owned(),
            ResponseError::EmailBuilderError(_) => "internal server error".to_owned()
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
            ResponseError::CustomFieldNotFound(_) => StatusCode::NOT_FOUND,
            ResponseError::TagNotFound(_) => StatusCode::NOT_FOUND,
            ResponseError::EntryMarkerNotFound(_) => StatusCode::NOT_FOUND,

            ResponseError::UsernameExists(_) => StatusCode::BAD_REQUEST,
            ResponseError::EmailExists(_) => StatusCode::BAD_REQUEST,
            ResponseError::EntryExists(_) => StatusCode::BAD_REQUEST,
            ResponseError::CustomFieldExists(_) => StatusCode::BAD_REQUEST,

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

            ResponseError::UuidError(_) => StatusCode::INTERNAL_SERVER_ERROR,

            ResponseError::EmailSmtpError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ResponseError::EmailBuilderError(_) => StatusCode::INTERNAL_SERVER_ERROR
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

impl From<lettre::transport::smtp::Error> for ResponseError {

    fn from(error: lettre::transport::smtp::Error) -> Self {
        ResponseError::EmailSmtpError(error)
    }
    
}

impl From<lettre::error::Error> for ResponseError {

    fn from(error: lettre::error::Error) -> Self {
        ResponseError::EmailBuilderError(error)
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

impl From<db::error::DbError> for ResponseError {

    fn from(error: db::error::DbError) -> Self {
        match error {
            db::error::DbError::Validation(msg) => ResponseError::Validation(msg),
            db::error::DbError::Postgres(err) => ResponseError::PostgresError(err)
        }
    }
    
}

#[derive(Debug)]
pub struct Error {
    code: StatusCode,
    r#type: String,
    msg: String
}

impl Error {

    pub fn new<C,T,M>(c: C, t: T, m: M) -> Error
    where
        C: Into<StatusCode>,
        T: Into<String>,
        M: Into<String>
    {
        Error {
            code: c.into(),
            r#type: t.into(),
            msg: m.into()
        }
    }
    
    pub fn get_code(&self) -> StatusCode {
        self.code
    }

    pub fn get_type(&self) -> String {
        self.r#type.clone()
    }

    pub fn get_msg(&self) -> String {
        self.msg.clone()
    }
}

impl fmt::Display for Error {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_msg())
    }

}

impl ActixResponseError for Error {

    fn error_response(&self) -> HttpResponse {
        response::json::respond_json(
            self.get_code(),
            response::json::ErrorJSON::build(
                self.get_msg(), 
                self.get_type()
            )
        )
    }
    
    fn status_code(&self) -> StatusCode {
        self.get_code()
    }
}

pub fn internal_error<M, T>(m: Option<M>, t: Option<T>) -> Error
where
    M: Into<String>,
    T: Into<String>
{
    if let Some(t) = t {
        if let Some(m) = m {
            Error::new(StatusCode::INTERNAL_SERVER_ERROR, t, m)
        } else {
            Error::new(StatusCode::INTERNAL_SERVER_ERROR, t, "internal server error")
        }
    } else {
        if let Some(m) = m {
            Error::new(StatusCode::INTERNAL_SERVER_ERROR, "InternalServerError", m)
        } else {
            Error::new(StatusCode::INTERNAL_SERVER_ERROR, "InternalServerError", "internal server error")
        }
    }
}

pub fn unauthorized<M, T>(m: Option<M>, t: Option<T>) -> Error
where
    M: Into<String>,
    T: Into<String>
{
    if let Some(t) = t {
        if let Some(m) = m {
            Error::new(StatusCode::UNAUTHORIZED, t, m)
        } else {
            Error::new(StatusCode::UNAUTHORIZED, t, "unauthorized access")
        }
    } else {
        if let Some(m) = m {
            Error::new(StatusCode::UNAUTHORIZED, "Unauthorized", m)
        } else {
            Error::new(StatusCode::UNAUTHORIZED, "Unauthorized", "unauthorized access")
        }
    }
}

pub fn bad_request<M, T>(m: Option<M>, t: Option<T>) -> Error
where
    M: Into<String>,
    T: Into<String>
{
    if let Some(t) = t {
        if let Some(m) = m {
            Error::new(StatusCode::BAD_REQUEST, t, m)
        } else {
            Error::new(StatusCode::BAD_REQUEST, t, "bad request")
        }
    } else {
        if let Some(m) = m {
            Error::new(StatusCode::BAD_REQUEST, "BadRequest", m)
        } else {
            Error::new(StatusCode::BAD_REQUEST, "BadRequest", "bad request")
        }
    }
}

pub fn not_found<M, T>(m: Option<M>, t: Option<T>) -> Error
where
    M: Into<String>,
    T: Into<String>
{
    if let Some(t) = t {
        if let Some(m) = m {
            Error::new(StatusCode::NOT_FOUND, t, m)
        } else {
            Error::new(StatusCode::NOT_FOUND, t, "not found")
        }
    } else {
        if let Some(m) = m {
            Error::new(StatusCode::NOT_FOUND, "NotFound", m)
        } else {
            Error::new(StatusCode::NOT_FOUND, "NotFound", "not found")
        }
    }
}

pub fn session_error() -> Error {
    unauthorized(Some("no session found for request"), Some("SessionError"))
}

pub fn invalid_password_error() -> Error{
    unauthorized(Some("invalid password given for account"), Some("InvalidPassword"))
}

pub fn permission_denied_error<M>(m: M) -> Error
where
    M: Into<String> 
{
    unauthorized(Some(m), Some("PermissionDenied"))
}

pub fn validation_error<M>(m: M) -> Error
where
    M: Into<String>
{
    bad_request(Some(m), Some("ValidationError"))
}

pub fn username_not_found(username: String) -> Error {
    not_found(Some(format!("failed to find the requested username: {}", username)), Some("UsernameNotFound"))
}

pub fn user_id_not_found(id: i32) -> Error {
    not_found(Some(format!("failed to find the requested user id: {}", id)), Some("UserIdNotFound"))
}

pub fn entry_not_found(id: i32) -> Error {
    not_found(Some(format!("failed to find the requested entry id: {}", id)), Some("EntryNotFound"))
}

pub fn text_entry_not_found(id: i32) -> Error {
    not_found(Some(format!("failed to find the requested text entry id: {}", id)), Some("TextEntryNotFound"))
}

pub fn custom_field_not_found(id: i32) -> Error {
    not_found(Some(format!("failed to find the requested custom field id: {}", id)), Some("CustomFieldNotFound"))
}

pub fn tag_not_found(id: i32) -> Error {
    not_found(Some(format!("failed to find the requested tag id: {}", id)), Some("TagNotFound"))
}

pub fn entry_marker_not_found(id: i32) -> Error {
    not_found(Some(format!("failed to find the requested entry marker id: {}", id)), Some("EntryMarkerNotFound"))
}

pub fn username_exists() -> Error {
    bad_request(Some("given username already exists"), Some("UsernameExists"))
}

pub fn email_exists() -> Error {
    bad_request(Some("given email already exists"), Some("EmailExists"))
}

pub fn entry_exists(day: String) -> Error {
    bad_request(Some(format!("given entry date already exists: {}", day)), Some("EntryExists"))
}

pub fn custom_field_exists(name: String) -> Error {
    bad_request(Some(format!("given custom field already exists: {}", name)), Some("CustomFieldExists"))
}

// From implementations

impl From<actix_web::error::Error> for Error {

    fn from(_error: actix_web::error::Error) -> Self {
        internal_error::<String,String>(None, None)
    }
    
}

impl From<actix_web::http::header::ToStrError> for Error {

    fn from(error: actix_web::http::header::ToStrError) -> Self {
        internal_error::<String,String>(None, None)
    }
    
}

impl From<tokio_postgres::Error> for Error {

    fn from(error: tokio_postgres::Error) -> Self {
        internal_error::<String,String>(None, None)
    }
    
}

impl From<bb8_postgres::bb8::RunError<tokio_postgres::Error>> for Error {

    fn from(error: bb8_postgres::bb8::RunError<tokio_postgres::Error>) -> Self {
        internal_error::<String,String>(None, None)
    }
    
}

impl From<argon2::Error> for Error {

    fn from(error: argon2::Error) -> Self {
        internal_error::<String,String>(None, None)
    }
    
}

impl From<openssl::error::Error> for Error {

    fn from(error: openssl::error::Error) -> Self {
        internal_error::<String,String>(None, None)
    }
    
}

impl From<openssl::error::ErrorStack> for Error {
    
    fn from(error: openssl::error::ErrorStack) -> Self {
        internal_error::<String,String>(None, None)
    }
    
}

impl From<uuid::Error> for Error {

    fn from(error: uuid::Error) -> Self {
        internal_error::<String,String>(None, None)
    }
    
}

impl From<lettre::transport::smtp::Error> for Error {

    fn from(error: lettre::transport::smtp::Error) -> Self {
        internal_error::<String,String>(None, None)
    }
    
}

impl From<lettre::error::Error> for Error {

    fn from(error: lettre::error::Error) -> Self {
        internal_error::<String,String>(None, None)
    }
    
}

impl From<std::fmt::Error> for Error {

    fn from(error: std::fmt::Error) -> Self {
        internal_error::<String,String>(None, None)
    }
    
}

impl From<std::io::Error> for Error {

    fn from(error: std::io::Error) -> Self {
        internal_error::<String,String>(None, None)
    }
    
}

impl From<serde_json::Error> for Error {
    
    fn from(error: serde_json::Error) -> Self {
        internal_error::<String,String>(None, None)
    }
    
}

impl From<db::error::DbError> for Error {

    fn from(error: db::error::DbError) -> Self {
        match error {
            db::error::DbError::Validation(msg) => validation_error(msg),
            db::error::DbError::Postgres(err) => internal_error(Some(format!("database error")), Some("DatabaseError"))
        }
    }
    
}