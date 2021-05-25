use tokio_postgres::{GenericClient, Error as PGError};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: i32,
    pub title: String,
    pub owner: i32,
    pub color: String,
    pub comment: Option<String>
}

pub async fn get_via_id(
    conn: &impl GenericClient,
    id: i32
) -> Result<Option<Tag>, PGError> {
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

pub async fn get_via_owner(
    conn: &impl GenericClient,
    owner: i32
) -> Result<Vec<Tag>, PGError> {
    let result = conn.query(
        "select id, title, color, owner, comment from tags where owner = $1",
        &[&owner]
    ).await?;
    let mut rtn: Vec<Tag> = Vec::with_capacity(result.len());

    for row in result {
        rtn.push(Tag {
            id: row.get(0),
            title: row.get(1),
            color: row.get(2),
            owner: row.get(3),
            comment: row.get(4)
        });
    }

    Ok(rtn)
}