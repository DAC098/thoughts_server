use std::convert::{From};

use crate::config;

pub type Result = std::result::Result<i32, AppError>;

#[derive(Debug)]
pub enum AppError {
    CliError(String),
    SslError(String),
    ConfigError(config::error::ConfigError),

    IoError(std::io::Error),

    DatabaseError(tokio_postgres::Error)
}

impl AppError {

    pub fn get_code(&self) -> i32 {
        match &*self {
            AppError::CliError(_) => 1,
            AppError::SslError(_) => 1,
            AppError::ConfigError(_) => 1,
            AppError::IoError(_) => 1,
            AppError::DatabaseError(_) => 1,
        }
    }

    pub fn get_msg(&self) -> String {
        match &*self {
            AppError::CliError(msg) => format!("AppError::CliError: {}", msg),
            AppError::SslError(msg) => format!("AppError::SslError: {}", msg),
            AppError::ConfigError(msg) => format!("AppError::ConfigError: {}", msg),
            AppError::IoError(io_error) => format!("AppError::IoError: {:?}", io_error),
            AppError::DatabaseError(db_error) => format!("AppError::DatabaseError: {:?}", db_error)
        }
    }

    pub fn get(self) -> (i32, String) {
        (self.get_code(), self.get_msg())
    }
}

impl From<std::io::Error> for AppError {

    fn from(error: std::io::Error) -> Self {
        AppError::IoError(error)
    }
    
}

impl From<config::error::ConfigError> for AppError {

    fn from(error: config::error::ConfigError) -> Self {
        AppError::ConfigError(error)
    }

}

impl From<tokio_postgres::Error> for AppError {

    fn from(error: tokio_postgres::Error) -> Self {
        AppError::DatabaseError(error)
    }
    
}