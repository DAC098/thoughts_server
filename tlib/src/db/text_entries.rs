use std::fmt::{Write};

use tokio_postgres::{GenericClient};
use serde::{Serialize, Deserialize};

use crate::db::error;

#[derive(Serialize, Deserialize)]
pub struct TextEntry {
    pub id: i32,
    pub thought: String,
    pub private: bool,
    pub entry: i32
}

pub async fn find_from_entry(
    conn: &impl GenericClient,
    entry: &i32,
    is_private: &Option<bool>,
) -> error::Result<Vec<TextEntry>> {
    let mut query = "\
    select id, \
           thought, \
           private, \
           entry
    from text_entries \
    where entry = $1".to_owned();

    if let Some(value) = is_private {
        write!(&mut query, " and private = {}", if *value { "true" } else { "false" })?;
    }

    write!(&mut query, " order by id")?;

    Ok(
        conn.query(
            query.as_str(),
            &[&entry]
        )
        .await?
        .iter()
        .map(|row| TextEntry {
            id: row.get(0),
            thought: row.get(1),
            private: row.get(2),
            entry: row.get(3)
        })
        .collect()
    )
}