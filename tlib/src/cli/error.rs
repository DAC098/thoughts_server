use std::fmt;
use std::ffi::OsString;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    General(String),

    IncompleteArg,
    UnknownArg(String),
    MissingArg(String),
    MissingArgValue(String),
    InvalidArg(String),

    FileNotFound(String),

    InvalidFile(OsString)
}

impl Error {

    pub fn get_msg(&self) -> String {
        match &*self {
            Error::General(msg) => msg.clone(),
            Error::IncompleteArg => format!("incomplete argument given."),
            Error::UnknownArg(arg) => format!("unknown argument given. {}", arg),
            Error::MissingArg(arg) => format!("missing argument: {}", arg),
            Error::MissingArgValue(arg) => format!("missing argument value for: {}", arg),
            Error::InvalidArg(msg) => msg.clone(),
            Error::FileNotFound(arg) => format!("failed to locate given file. {}", arg),
            Error::InvalidFile(arg) => format!("specified configuration file is not a file. {:?}", arg)
        }
    }
    
    pub fn get_code(&self) -> i32 {
        match &*self {
            Error::General(_) |
            Error::IncompleteArg |
            Error::UnknownArg(_) |
            Error::MissingArg(_) |
            Error::MissingArgValue(_) |
            Error::InvalidArg(_) |
            Error::FileNotFound(_) |
            Error::InvalidFile(_) => 1
        }
    }

    pub fn into_tuple(self) -> (i32, String) {
        (self.get_code(), self.get_msg())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_msg())
    }
}