use tokio_postgres::{GenericClient};
use chrono::{DateTime, Utc};
use chrono::serde::{ts_seconds, ts_seconds_option};
use serde::{Serialize, Deserialize};

use crate::db::error;

#[derive(Serialize, Deserialize)]
pub struct EntryComment {
    pub id: i32,

    pub entry: i32,
    pub owner: i32,
    pub comment: String,

    #[serde(with = "ts_seconds")]
    pub created: DateTime<Utc>,

    #[serde(with = "ts_seconds_option")]
    pub updated: Option<DateTime<Utc>>
}

// pub async fn find_from_entry(
//     conn: &impl GenericClient,
//     entry: &i32,
// ) -> error::Result<Vec<EntryComment>> {
//     Ok(conn.query(
//         "\
//         select id, \
//                entry, \
//                owner, \
//                comment, \
//                created, \
//                updated \
//         from entry_comments \
//         where entry = $1",
//         &[entry]
//     )
//     .await?
//     .iter()
//     .map(|row| EntryComment {
//         id: row.get(0),
//         entry: row.get(1),
//         owner: row.get(2),
//         comment: row.get(3),
//         created: row.get(4),
//         updated: row.get(5)
//     })
//     .collect())
// }

pub async fn find_from_id(
    conn: &impl GenericClient,
    id: &i32,
) -> error::Result<Option<EntryComment>> {
    let mut result = conn.query(
        "\
        select id, \
               entry, \
               owner, \
               comment, \
               created, \
               updated \
        from entry_comments \
        where id = $1",
        &[id]
    ).await?;

    if result.len() == 1 {
        let row = result.pop().unwrap();

        Ok(Some(EntryComment {
            id: row.get(0),
            entry: row.get(1),
            owner: row.get(2),
            comment: row.get(3),
            created: row.get(4),
            updated: row.get(5)
        }))
    } else {
        Ok(None)
    }
}