use std::fmt::{Write};

use tokio_postgres::{GenericClient};
use serde::{Serialize, Deserialize};

use crate::db::{error, query};

#[derive(Serialize, Deserialize)]
pub struct AudioEntry {
    pub id: i32,
    pub private: bool,
    pub entry: i32
}

pub async fn find_from_id(
    conn: &impl GenericClient,
    audio_id: &i32
) -> error::Result<Option<AudioEntry>> {
    if let Some(row) = conn.query_opt(
        "\
        select id, \
               private, \
               entry \
        from audio_entries \
        where id = $1",
        &[audio_id]
    ).await? {
        Ok(Some(AudioEntry {
            id: row.get(0),
            private: row.get(1),
            entry: row.get(2)
        }))
    } else {
        Ok(None)
    }
}

pub async fn find_from_entry(
    conn: &impl GenericClient,
    entry_id: &i32,
    is_private: &Option<bool>
) -> error::Result<Vec<AudioEntry>> {
    let mut query_str = format!("\
    select id, \
           private, \
           entry \
    from audio_entries \
    where entry_id = $1");
    let mut query_slice = query::QueryParams::with_capacity(1);
    query_slice.push(entry_id);

    if let Some(private) = is_private {
        write!(&mut query_str, " and private = ${}", query_slice.push(private))?;
    }

    Ok(
        conn.query(query_str.as_str(), query_slice.slice())
        .await?
        .iter()
        .map(|row| AudioEntry {
            id: row.get(0),
            private: row.get(1),
            entry: row.get(2)
        })
        .collect()
    )
}