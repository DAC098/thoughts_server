use std::{fmt};
use std::convert::{From};

use actix_web::{
    error::ResponseError as ActixResponseError, 
    http::StatusCode, 
    HttpResponse
};

use tlib::{db};

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
    GlobalCustomFieldNotFound(i32),
    TagNotFound(i32),
    EntryMarkerNotFound(i32),

    UsernameExists(String),
    EmailExists(String),
    EntryExists(String),
    CustomFieldExists(String),
    GlobalCustomFieldExists(String),

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
    EmailBuilderError(lettre::error::Error),

    ChronoParserError(chrono::ParseError),
}

impl ResponseError {

    fn error_type(&self) -> &str {
        match &*self {

            ResponseError::Session => "Session",
            ResponseError::InvalidPassword => "InvalidPassword",

            ResponseError::PermissionDenied(_) => "PermissionDenied",
            ResponseError::Validation(_) => "Validation",

            ResponseError::UsernameNotFound(_) => "UsernameNotFound",
            ResponseError::UserIDNotFound(_) => "UserIDNotFound",
            ResponseError::EntryNotFound(_) => "EntryNotFound",
            ResponseError::TextEntryNotFound(_) => "TextEntryNotFound",
            ResponseError::CustomFieldNotFound(_) => "CustomFieldNotFound",
            ResponseError::GlobalCustomFieldNotFound(_) => "GlobalCustomFieldNotFound",
            ResponseError::TagNotFound(_) => "TagNotFound",
            ResponseError::EntryMarkerNotFound(_) => "EntryMarkerNotFound",

            ResponseError::UsernameExists(_) => "UsernameExists",
            ResponseError::EmailExists(_) => "EmailExists",
            ResponseError::EntryExists(_) => "EntryExists",
            ResponseError::CustomFieldExists(_) => "CustomFieldExists",
            ResponseError::GlobalCustomFieldExists(_) => "GlobalCustomFieldExists",

            ResponseError::PostgresError(_) |
            ResponseError::BB8Error(_) => "DatabaseError",

            ResponseError::RustFMTError(_) |
            ResponseError::RustIOError(_) |

            ResponseError::SerdeJsonError(_) |

            ResponseError::ActixError(_) |
            ResponseError::HeaderError(_) |

            ResponseError::Argon2Error(_) |

            ResponseError::OpensslError(_) |
            ResponseError::OpensslErrorStack(_) |

            ResponseError::UuidError(_) |

            ResponseError::EmailSmtpError(_) |
            ResponseError::EmailBuilderError(_) |
            
            ResponseError::ChronoParserError(_) => "InternalError"
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
            ResponseError::GlobalCustomFieldNotFound(id) => format!("failed to find the request global custom field id: {}", id),
            ResponseError::CustomFieldNotFound(id) => format!("failed to find the requested custom field id: {}", id),
            ResponseError::TagNotFound(id) => format!("failed to find the requested tag id: {}", id),
            ResponseError::EntryMarkerNotFound(id) => format!("failed to find the requested marker id: {}", id),

            ResponseError::UsernameExists(_) => format!("given username already exist"),
            ResponseError::EmailExists(_) => format!("given email already exists"),
            ResponseError::EntryExists(created) => format!("given entry date already exists. date: {}", created),
            ResponseError::CustomFieldExists(name) => format!("given custom field already exists. name: {}", name),
            ResponseError::GlobalCustomFieldExists(name) => format!("given global custom field already exists. name: {}", name),

            ResponseError::PostgresError(_) |
            ResponseError::BB8Error(_) => "database server error".to_owned(),

            ResponseError::RustFMTError(_) |
            ResponseError::RustIOError(_) |

            ResponseError::SerdeJsonError(_) |

            ResponseError::ActixError(_) |
            ResponseError::HeaderError(_) |

            ResponseError::Argon2Error(_) |

            ResponseError::OpensslError(_) |
            ResponseError::OpensslErrorStack(_) |

            ResponseError::UuidError(_) |

            ResponseError::EmailSmtpError(_) |
            ResponseError::EmailBuilderError(_) |
            
            ResponseError::ChronoParserError(_) => "internal server error".to_owned()
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

            ResponseError::Session |
            ResponseError::InvalidPassword |
            ResponseError::PermissionDenied(_) => StatusCode::UNAUTHORIZED,

            ResponseError::UsernameNotFound(_) |
            ResponseError::UserIDNotFound(_) |
            ResponseError::EntryNotFound(_) |
            ResponseError::TextEntryNotFound(_) |
            ResponseError::CustomFieldNotFound(_) |
            ResponseError::GlobalCustomFieldNotFound(_) |
            ResponseError::TagNotFound(_) |
            ResponseError::EntryMarkerNotFound(_) => StatusCode::NOT_FOUND,

            ResponseError::Validation(_) |
            ResponseError::UsernameExists(_) |
            ResponseError::EmailExists(_) |
            ResponseError::EntryExists(_) |
            ResponseError::CustomFieldExists(_) |
            ResponseError::GlobalCustomFieldExists(_) => StatusCode::BAD_REQUEST,

            ResponseError::RustFMTError(_) |
            ResponseError::RustIOError(_) |

            ResponseError::SerdeJsonError(_) |

            ResponseError::ActixError(_) |
            ResponseError::HeaderError(_) |

            ResponseError::PostgresError(_) |
            ResponseError::BB8Error(_) |

            ResponseError::Argon2Error(_) |

            ResponseError::OpensslError(_) |
            ResponseError::OpensslErrorStack(_) |

            ResponseError::UuidError(_) |

            ResponseError::EmailSmtpError(_) |
            ResponseError::EmailBuilderError(_) |

            ResponseError::ChronoParserError(_) => StatusCode::INTERNAL_SERVER_ERROR
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

impl From<db::error::Error> for ResponseError {

    fn from(error: db::error::Error) -> Self {
        match error {
            db::error::Error::Validation(msg) => ResponseError::Validation(msg),
            db::error::Error::Postgres(err) => ResponseError::PostgresError(err),
            db::error::Error::RustFmt(err) => ResponseError::RustFMTError(err)
        }
    }
    
}

impl From<chrono::ParseError> for ResponseError {

    fn from(error: chrono::ParseError) -> Self {
        ResponseError::ChronoParserError(error)
    }

}