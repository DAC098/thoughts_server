use std::fmt;
use std::convert::{From};

#[derive(Debug)]
pub enum DbError {
    Validation(String),

    Postgres(tokio_postgres::Error)
}

pub type Result<T> = std::result::Result<T, DbError>;

impl DbError {

    pub fn get_type(&self) -> String {
        match &*self {
            DbError::Validation(_) => "DbError::Validation".to_owned(),
            DbError::Postgres(_) => "DbError::Postgres".to_owned()
        }
    }
    
}

impl fmt::Display for DbError {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_type())?;

        match &*self {
            DbError::Validation(msg) => write!(f, " {}", msg),
            DbError::Postgres(err) => write!(f, " {:?}", err)
        }
    }
    
}

impl From<tokio_postgres::Error> for DbError {

    fn from(error: tokio_postgres::Error) -> Self {
        DbError::Postgres(error)
    }
    
}