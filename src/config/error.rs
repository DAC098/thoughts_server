use std::convert::{From};
use std::ffi::{OsString};

#[derive(Debug)]
pub enum ConfigError {
    UnknownFileExtension,
    InvalidFileExtension(OsString),
    NotReadableFile(OsString),
    
    JsonError(serde_json::Error),
    YamlError(serde_yaml::Error),

    IOError(std::io::Error)
}

pub type Result<T> = std::result::Result<T, ConfigError>;

impl From<serde_json::Error> for ConfigError {
    fn from(error: serde_json::Error) -> Self {
        ConfigError::JsonError(error)
    }
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(error: serde_yaml::Error) -> Self {
        ConfigError::YamlError(error)
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        ConfigError::IOError(error)
    }
}