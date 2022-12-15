use lazy_static::lazy_static;
use regex::Regex;
use chrono::{DateTime, Utc, ParseResult, NaiveDateTime};

lazy_static! {
    static ref TAGS_REG: Regex = Regex::new(r"(\d+),?").unwrap();
}

pub fn get_date(from: &Option<String>) -> ParseResult<Option<DateTime<Utc>>> {
    if let Some(ref_from) = from.as_ref() {
        if let Ok(int) = i64::from_str_radix(ref_from.as_str(), 10) {
            Ok(Some(DateTime::from_utc(NaiveDateTime::from_timestamp(int, 0), Utc)))
        } else {
            Ok(Some(
                DateTime::parse_from_rfc3339(ref_from.as_str())?.with_timezone(&Utc)
            ))
        }
    } else {
        Ok(None)
    }
}

pub fn get_tags(tags: &Option<String>) -> Option<Vec<i32>> {
    if let Some(ref_tags) = tags.as_ref() {
        let mut rtn: Vec<i32> = Vec::new();

        for split in ref_tags.split(",") {
            if let Ok(tag) = i32::from_str_radix(split, 10) {
                rtn.push(tag);
            }
        }

        Some(rtn)
    } else {
        None
    }
}