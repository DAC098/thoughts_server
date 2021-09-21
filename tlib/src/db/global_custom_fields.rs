use tokio_postgres::{GenericClient};
use serde::{Serialize, Deserialize};

use crate::db::{custom_fields, error};

#[derive(Serialize, Deserialize)]
pub struct GlobalCustomField {
    pub id: i32,
    pub name: String,
    pub comment: Option<String>,
    pub config: custom_fields::CustomFieldType
}

pub async fn find_from_id(
    conn: &impl GenericClient,
    id: &i32
) -> error::Result<Option<GlobalCustomField>> {
    if let Some(row) = conn.query_opt(
        "\
        select id, \
               name, \
               comment, \
               config \
        from global_custom_fields \
        where id = $1",
        &[id]
    ).await? {
        Ok(Some(GlobalCustomField {
            id: row.get(0),
            name: row.get(1),
            comment: row.get(2),
            config: serde_json::from_value(row.get(3)).unwrap()
        }))
    } else {
        Ok(None)
    }
}

pub async fn find_all(
    conn: &impl GenericClient,
) -> error::Result<Vec<GlobalCustomField>> {
    Ok(
        conn.query(
            "\
            select id, \
                   name, \
                   comment, \
                   config \
            from global_custom_fields", 
            &[]
        )
        .await?
        .iter()
        .map(|row| GlobalCustomField {
            id: row.get(0),
            name: row.get(1),
            comment: row.get(2),
            config: serde_json::from_value(row.get(3)).unwrap()
        })
        .collect()
    )
}