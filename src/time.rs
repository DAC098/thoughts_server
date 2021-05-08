use chrono::{Local};

pub fn now_rfc3339() -> String {
    Local::now().to_rfc3339()
}