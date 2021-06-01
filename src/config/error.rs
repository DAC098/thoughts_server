use std::{fmt};
use std::convert::{From};
use std::ffi::{OsString};

#[derive(Debug)]
pub enum ConfigError {
    InvalidConfig(String),

    UnknownFileExtension,
    InvalidFileExtension(OsString),
    
    JsonError(serde_json::Error),
    YamlError(serde_yaml::Error),

    IOError(std::io::Error)
}

pub type Result<T> = std::result::Result<T, ConfigError>;

impl ConfigError {

    pub fn get_msg(&self) -> String {
        match &*self {
            ConfigError::InvalidConfig(msg) => format!("{}", msg),
            ConfigError::UnknownFileExtension => format!("unknown file extension given"),
            ConfigError::InvalidFileExtension(ext) => format!("invalid file extension given. {:?}", ext),
            ConfigError::JsonError(err) => {
                match err.classify() {
                    serde_json::error::Category::Io => format!(
                        "json io error"
                    ),
                    serde_json::error::Category::Syntax => format!(
                        "json syntax error {}:{}", err.line(), err.column()
                    ),
                    serde_json::error::Category::Data => format!(
                        "json data error"
                    ),
                    serde_json::error::Category::Eof => format!(
                        "json eof error"
                    )
                }
            },
            ConfigError::YamlError(err) => {
                if let Some(location) = err.location() {
                    format!("yaml error {}:{}", location.line(), location.column())
                } else {
                    format!("yaml error")
                }
            },
            ConfigError::IOError(err) => format!("{:?}", err)
        }
    }
    
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_msg())
    }
}

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