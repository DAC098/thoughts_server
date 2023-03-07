use tokio_postgres::{GenericClient};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use chrono::serde::{ts_seconds};

use crate::db::{error};

#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub id: i32,
    #[serde(with = "ts_seconds")]
    pub day: DateTime<Utc>,
    pub created: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
    pub deleted: Option<DateTime<Utc>>,
    pub owner: i32,
}

/// finds an entry based on the entry id
pub async fn find_from_id(
    conn: &impl GenericClient,
    id: &i32
) -> error::Result<Option<Entry>> {
    if let Some(result) = conn.query_opt(
        "\
        select id, \
               day, \
               created, \
               updated, \
               deleted, \
               owner \
        from entries \
        where id = $1",
        &[id]
    ).await? {
        Ok(Some(Entry {
            id: result.get(0),
            day: result.get(1),
            created: result.get(2),
            updated: result.get(3),
            deleted: result.get(4),
            owner: result.get(5)
        }))
    } else {
        Ok(None)
    }
}

/// finds an entry based on the user id and entry id
pub async fn from_user_and_id(
    conn: &impl GenericClient,
    user_id: &i32,
    id: &i32
) -> error::Result<Option<Entry>> {
    if let Some(record) = conn.query_opt(
        "\
        select id, \
               day, \
               created, \
               updated, \
               deleted, \
               owner \
        from entries \
        where owner = $1 and \
              id = $2",
        &[user_id, id]
    ).await? {
        Ok(Some(Entry {
            id: record.get(0),
            day: record.get(1),
            created: record.get(2),
            updated: record.get(3),
            deleted: record.get(4),
            owner: record.get(5)
        }))
    } else {
        Ok(None)
    }
}
