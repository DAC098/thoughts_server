use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use chrono::{DateTime, Utc, ParseResult, NaiveDateTime};

lazy_static! {
    static ref TAGS_REG: Regex = Regex::new(r"(\d+),?").unwrap();
}

#[derive(Deserialize)]
pub struct QueryEntries {
    from: Option<String>,
    to: Option<String>,
    tags: Option<String>,
}

impl QueryEntries {

    pub fn get_from(&self) -> ParseResult<Option<DateTime<Utc>>> {
        if let Some(ref_from) = self.from.as_ref() {
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

    pub fn get_to(&self) -> ParseResult<Option<DateTime<Utc>>> {
        if let Some(ref_to) = self.to.as_ref() {
            if let Ok(int) = i64::from_str_radix(ref_to.as_str(), 10) {
                Ok(Some(DateTime::from_utc(NaiveDateTime::from_timestamp(int, 0), Utc)))
            } else {
                Ok(Some(
                    DateTime::parse_from_rfc3339(ref_to.as_str())?.with_timezone(&Utc)
                ))
            }
        } else {
            Ok(None)
        }
    }
    
    pub fn get_tags(&self) -> Option<Vec<i32>> {
        if let Some(ref_tags) = self.tags.as_ref() {
            let mut rtn: Vec<i32> = Vec::new();

            for capture in TAGS_REG.captures_iter(ref_tags.as_str()) {
                if let Ok(tag) = i32::from_str_radix(&capture[1], 10) {
                    rtn.push(tag);
                }
            }

            Some(rtn)
        } else {
            None
        }
    }
}