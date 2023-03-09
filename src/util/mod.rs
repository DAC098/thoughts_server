use std::fmt::{Write};

pub mod time;
pub mod string;
pub mod file;

/// clones the internal value of an option and returns a new option
#[allow(dead_code)]
#[inline]
pub fn clone_option<T>(opt: &Option<T>) -> Option<T>
where
    T: Clone
{
    match opt {
        Some(t) => Some(t.clone()),
        None => None
    }
}

pub fn hex_string<T>(slice: T) -> Result<String, std::fmt::Error>
where
    T: AsRef<[u8]>
{
    let mut rtn = String::with_capacity(slice.as_ref().len() * 2);

    for byte in slice.as_ref() {
        write!(rtn, "{:02x}", byte)?;
    }

    Ok(rtn)
}
