use std::fmt::{Write};

pub mod time;

pub fn clone_option<T>(opt: &Option<T>) -> Option<T>
where
    T: Clone
{
    match opt {
        Some(t) => Some(t.clone()),
        None => None
    }
}

pub fn hex_string(slice: &[u8]) -> Result<String, std::fmt::Error> {
    let mut rtn = String::with_capacity(slice.len() * 2);

    for byte in slice {
        write!(rtn, "{:02x}", byte)?;
    }

    Ok(rtn)
}