use std::{fmt};
use std::convert::{From};
use std::ffi::{OsString};

#[derive(Debug)]
pub enum Error {
    InvalidConfig(String),

    UnknownFileExtension,
    InvalidFileExtension(OsString),
    
    JsonError(serde_json::Error),
    YamlError(serde_yaml::Error),

    IOError(std::io::Error)
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {

    pub fn get_msg(&self) -> String {
        match &*self {
            Error::InvalidConfig(msg) => format!("{}", msg),
            Error::UnknownFileExtension => format!("unknown file extension given"),
            Error::InvalidFileExtension(ext) => format!("invalid file extension given. {:?}", ext),
            Error::JsonError(err) => {
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
            Error::YamlError(err) => {
                if let Some(location) = err.location() {
                    format!("yaml error {}:{}", location.line(), location.column())
                } else {
                    format!("yaml error")
                }
            },
            Error::IOError(err) => format!("{:?}", err)
        }
    }
    
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_msg())
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::JsonError(error)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(error: serde_yaml::Error) -> Self {
        Error::YamlError(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IOError(error)
    }
}