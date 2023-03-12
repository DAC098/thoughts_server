type BoxDynError = Box<dyn std::error::Error>;

#[derive(Debug)]
pub struct Error {
    msg: String,
    src: Option<BoxDynError>,
}

pub type Result<T> = std::result::Result<T, Error>;

/*
impl<T, E> From<std::result::Result<T, E>> for Result<T>
where
    E: Into<Error>
{
    fn from(res: std::result::Result<T, E>) -> Result<T> {
        res.map_err(|e| e.into())
    }
}
*/

impl Error {
    pub fn new() -> Error {
        Error { 
            msg: String::from("application error"), 
            src: None 
        }
    }

    pub fn with_message<M>(mut self, msg: M) -> Error
    where
        M: Into<String>
    {
        self.msg = msg.into();
        self
    }

    pub fn with_source<S>(mut self, src: S) -> Error
    where
        S: Into<BoxDynError>
    {
        self.src = Some(src.into());
        self
    }

    pub fn message(&self) -> &String {
        &self.msg
    }

    pub fn source(&self) -> &Option<BoxDynError> {
        &self.src
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.msg)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.src.as_ref().map(|e| &**e as _)
    }
}

macro_rules! generic_catch {
    ($e:path) => {
        impl From<$e> for Error {
            fn from(err: $e) -> Self {
                Error::new()
                    .with_source(err)
            }
        }
    };
    ($e:path,$m:expr) => {
        impl From<$e> for Error {
            fn from(err: $e) -> Self {
                Error::new()
                    .with_message($m)
                    .with_source(err)
            }
        }
    }
}

generic_catch!(std::io::Error, "std::io::Error");
generic_catch!(postgres::Error);
generic_catch!(refinery::Error);
