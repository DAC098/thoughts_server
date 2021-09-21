use std::default::{Default};

use tokio_postgres::{GenericClient};
use chrono::{Datelike, DateTime, Utc, NaiveDate};
use serde::{Serialize, Deserialize};

use crate::db::error;

#[derive(Serialize, Deserialize)]
pub struct UserData {
    pub owner: i32,

    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,

    pub dob: NaiveDate
}

impl Default for UserData {
    fn default() -> Self {
        let now: DateTime<Utc> = Utc::now();

        UserData {
            owner: 0,
            prefix: None,
            suffix: None,
            first_name: String::new(),
            last_name: String::new(),
            middle_name: None,
            dob: NaiveDate::from_ymd(now.year(), now.month(), now.day())
        }
    }
}

pub async fn find_from_owner(
    conn: &impl GenericClient,
    owner: &i32
) -> error::Result<Option<UserData>> {
    let result = conn.query(
        "\
        select owner, \
               prefix, suffix, \
               first_name, last_name, middle_name, \
               dob \
        from user_data \
        where owner = $1",
        &[owner]
    ).await?;

    if result.len() == 1 {
        Ok(Some(UserData {
            owner: result[0].get(0),
            prefix: result[0].get(1),
            suffix: result[0].get(2),
            first_name: result[0].get(3),
            last_name: result[0].get(4),
            middle_name: result[0].get(5),
            dob: result[0].get(6)
        }))
    } else {
        Ok(None)
    }
}