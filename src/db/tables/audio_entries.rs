use std::fmt::{Write};

use tokio_postgres::{GenericClient};
use serde::{Serialize, Deserialize};

use crate::db::{error, query};

#[derive(Serialize, Deserialize)]
pub struct AudioEntry {
    pub id: i32,
    pub private: bool,
    pub comment: Option<String>,
    pub entry: i32,
    pub mime_type: String,
    pub mime_subtype: String,
    pub file_size: i64,
}

pub async fn find_from_id(
    conn: &impl GenericClient,
    audio_id: &i32,
    is_private: &Option<bool>,
) -> error::Result<Option<AudioEntry>> {
    let mut query_str = format!("\
    select id, \
           private, \
           entry, \
           mime_type, \
           mime_subtype, \
           file_size \
    from audio_entries \
    where id = $1");
    let mut query_slice = query::QueryParams::with_capacity(2);
    query_slice.push(audio_id);

    if let Some(private) = is_private {
        write!(&mut query_str, " and private = ${}", query_slice.push(private))?;
    }

    if let Some(row) = conn.query_opt(
        query_str.as_str(), 
        query_slice.slice()
    ).await? {
        Ok(Some(AudioEntry {
            id: row.get(0),
            private: row.get(1),
            comment: row.get(2),
            entry: row.get(3),
            mime_type: row.get(4),
            mime_subtype: row.get(5),
            file_size: row.get(6),
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
           comment, \
           entry, \
           mime_type, \
           mime_subtype, \
           file_size \
    from audio_entries \
    where entry_id = $1");
    let mut query_slice = query::QueryParams::with_capacity(1);
    query_slice.push(entry_id);

    if let Some(private) = is_private {
        write!(&mut query_str, " and private = ${}", query_slice.push(private))?;
    }

    Ok(conn.query(query_str.as_str(), query_slice.slice())
        .await?
        .iter()
        .map(|row| AudioEntry {
            id: row.get(0),
            private: row.get(1),
            comment: row.get(2),
            entry: row.get(3),
            mime_type: row.get(4),
            mime_subtype: row.get(5),
            file_size: row.get(6),
        })
        .collect())
}
