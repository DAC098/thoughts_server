use std::{fmt};
use std::ffi::{OsString};

pub type Result<T> = std::result::Result<T, CliError>;

#[derive(Debug)]
pub enum CliError {
    IncompleteArg,
    UnknownArg(String),

    FileNotFound(String),

    InvalidFile(OsString)
}

impl CliError {

    pub fn get_msg(&self) -> String {
        match &*self {
            CliError::IncompleteArg => format!("incomplete argument given."),
            CliError::UnknownArg(arg) => format!("unknown argument given. {}", arg),
            CliError::FileNotFound(arg) => format!("failed to locate given file. {}", arg),
            CliError::InvalidFile(arg) => format!("specified configuration file is not a file. {:?}", arg)
        }
    }
    
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_msg())
    }
}