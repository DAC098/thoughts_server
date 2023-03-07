use tokio_postgres::{GenericClient};
use serde::{Serialize, Deserialize};

use crate::db::error;

#[derive(Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: i32,
    pub title: String,
    pub owner: i32,
    pub color: String,
    pub comment: Option<String>
}

pub async fn find_from_id(
    conn: &impl GenericClient,
    id: i32
) -> error::Result<Option<Tag>> {
    let result = conn.query(
        "select id, title, color, owner, comment from tags where id = $1",
        &[&id]
    ).await?;

    if result.len() == 1 {
        Ok(Some(Tag {
            id: result[0].get(0),
            title: result[0].get(1),
            color: result[0].get(2),
            owner: result[0].get(3),
            comment: result[0].get(4)
        }))
    } else {
        Ok(None)
    }
}

pub async fn find_from_owner(
    conn: &impl GenericClient,
    owner: i32
) -> error::Result<Vec<Tag>> {
    Ok(
        conn.query(
            "select id, title, color, owner, comment from tags where owner = $1",
            &[&owner]
        )
        .await?
        .iter()
        .map(|row| Tag {
            id: row.get(0),
            title: row.get(1),
            color: row.get(2),
            owner: row.get(3),
            comment: row.get(4)
        })
        .collect()
    )
}
