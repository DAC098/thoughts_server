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
    pub owner: i32,
}

pub async fn find_from_id(
    conn: &impl GenericClient,
    id: &i32
) -> error::Result<Option<Entry>> {
    let result = conn.query(
        "\
        select id, day, owner \
        from entries \
        where id = $1",
        &[id]
    ).await?;

    if result.len() == 0 {
        Ok(None)
    } else {
        Ok(Some(Entry {
            id: result[0].get(0),
            day: result[0].get(1),
            owner: result[0].get(2)
        }))
    }
}