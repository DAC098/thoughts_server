use serde::Serialize;
use tokio_postgres::GenericClient;

use crate::db::error::Result;

#[derive(Serialize)]
pub struct Group {
    pub id: i32,
    pub name: String
}

pub async fn find_id(conn: &impl GenericClient, id: &i32) -> Result<Option<Group>> {
    if let Some(row) = conn.query_opt(
        "\
        select id, \
               name \
        from groups \
        where id = $1",
        &[id]
    ).await? {
        Ok(Some(Group {
            id: row.get(0),
            name: row.get(1)
        }))
    } else {
        Ok(None)
    }
}

pub async fn get_all(conn: &impl GenericClient) -> Result<Vec<Group>> {
    Ok(
        conn.query(
            "\
            select id, \
                   name \
            from groups",
            &[]
        ).await?
        .into_iter()
        .map(|row| Group {
            id: row.get(0),
            name: row.get(1)
        })
        .collect()
    )
}