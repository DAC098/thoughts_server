use chrono::{Local, NaiveDate};

pub fn now_rfc3339() -> String {
    Local::now().to_rfc3339()
}

pub fn naive_date_to_string(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}