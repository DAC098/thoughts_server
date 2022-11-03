use std::fmt;
use std::convert::From;
use actix_web::http::StatusCode;

use crate::net::http::error;

#[derive(Debug)]
pub enum Error {
    Validation(String),

    Postgres(tokio_postgres::Error),

    RustFmt(std::fmt::Error)
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {

    pub fn get_type(&self) -> String {
        match &*self {
            Error::Validation(_) => "Validation".to_owned(),
            Error::Postgres(_) => "Postgres".to_owned(),
            Error::RustFmt(_) => "RustFmt".to_owned()
        }
    }

    pub fn get_msg(&self) -> String {
        match &*self {
            Error::Validation(msg) => format!("{}", msg),
            Error::Postgres(err) => format!("{:?}", err),
            Error::RustFmt(err) => format!("{:?}", err)
        }
    }
    
}

impl fmt::Display for Error {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "db::error::{} {}", self.get_type(), self.get_msg())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Validation(_) => None,
            Error::Postgres(err) => Some(err),
            Error::RustFmt(err) => Some(err)
        }
    }
}

impl From<tokio_postgres::Error> for Error {

    fn from(error: tokio_postgres::Error) -> Self {
        Error::Postgres(error)
    }
    
}

impl From<std::fmt::Error> for Error {
    fn from(error: std::fmt::Error) -> Self {
        Error::RustFmt(error)
    }
}

impl From<Error> for error::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Validation(msg) => {
                error::Error::new()
                    .set_status(StatusCode::BAD_REQUEST)
                    .set_name("Validation")
                    .set_message(msg)
            },
            Error::Postgres(err) => {
                error::Error::new()
                    .set_name("DatabaseError")
                    .set_message("database error during request")
                    .set_source(err)
                },
            Error::RustFmt(err) => {
                error::Error::new()
                    .set_name("DatabaseError")
                    .set_message("database error during request")
                    .set_source(err)
            }
        }
    }
}