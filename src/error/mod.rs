use std::{fmt};
use std::convert::{From};

use tlib::{cli, config};

pub type Result = std::result::Result<i32, AppError>;

#[derive(Debug)]
pub enum AppError {
    CliError(cli::error::Error),
    SslError(String),
    ConfigError(config::error::Error),

    IoError(std::io::Error),

    DatabaseError(tokio_postgres::Error)
}

impl AppError {

    pub fn get_code(&self) -> i32 {
        match &*self {
            AppError::CliError(_) |
            AppError::SslError(_) |
            AppError::ConfigError(_) |
            AppError::IoError(_) |
            AppError::DatabaseError(_) => 1,
        }
    }

    pub fn get_msg(&self) -> String {
        match &*self {
            AppError::CliError(cli_error) => format!("AppError::CliError: {}", cli_error.get_msg()),
            AppError::SslError(msg) => format!("AppError::SslError: {}", msg),
            AppError::ConfigError(msg) => format!("AppError::ConfigError: {}", msg),
            AppError::IoError(io_error) => format!("AppError::IoError: {:?}", io_error),
            AppError::DatabaseError(db_error) => format!("AppError::DatabaseError: {:?}", db_error)
        }
    }
}

impl fmt::Display for AppError {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_msg())
    }
    
}

impl From<std::io::Error> for AppError {

    fn from(error: std::io::Error) -> Self {
        AppError::IoError(error)
    }
    
}

impl From<config::error::Error> for AppError {

    fn from(error: config::error::Error) -> Self {
        AppError::ConfigError(error)
    }

}

impl From<cli::error::Error> for AppError {

    fn from(error: cli::error::Error) -> Self {
        AppError::CliError(error)
    }
    
}

impl From<tokio_postgres::Error> for AppError {

    fn from(error: tokio_postgres::Error) -> Self {
        AppError::DatabaseError(error)
    }
    
}