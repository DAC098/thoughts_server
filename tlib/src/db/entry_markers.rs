use tokio_postgres::{GenericClient};
use serde::{Serialize, Deserialize};

use crate::db::error;

#[derive(Serialize, Deserialize)]
pub struct EntryMarker {
    pub id: i32,
    pub title: String,
    pub comment: Option<String>,
    pub entry: i32
}

pub async fn find_from_entry(
    conn: &impl GenericClient,
    entry: &i32
) -> error::Result<Vec<EntryMarker>> {
    Ok(
        conn.query(
            "
            select id, \
                   title, \
                   comment, \
                   entry \
            from entry_markers \
            where entry = $1", 
            &[&entry]
        )
        .await?
        .iter()
        .map(|row| EntryMarker {
            id: row.get(0),
            title: row.get(1),
            comment: row.get(2),
            entry: row.get(3)
        })
        .collect()
    )
}