use std::collections::{HashMap};

use tokio_postgres::{GenericClient};
use serde::{Serialize, Deserialize};

use crate::db::{
    error,
    entries,
    entry_markers,
    custom_field_entries,
    text_entries,
    entries2tags,
};

#[derive(Serialize, Deserialize)]
pub struct ComposedEntry {
    pub entry: entries::Entry,
    pub tags: Vec<i32>,
    pub markers: Vec<entry_markers::EntryMarker>,
    pub custom_field_entries: HashMap<i32, custom_field_entries::CustomFieldEntry>,
    pub text_entries: Vec<text_entries::TextEntry>
}

pub async fn find_from_entry_id(
    conn: &impl GenericClient,
    entry_id: &i32,
    is_private: &Option<bool>,
) -> error::Result<Option<ComposedEntry>> {
    if let Some(entry) = entries::find_from_id(conn, entry_id).await? {
        let results = custom_field_entries::find_from_entry(conn, entry_id).await?;
        let mut custom_field_map: HashMap<i32, custom_field_entries::CustomFieldEntry> = HashMap::with_capacity(results.len());

        for record in results {
            custom_field_map.insert(record.field, record);
        }

        Ok(Some(ComposedEntry {
            entry,
            tags: entries2tags::find_id_from_entry(conn, entry_id).await?,
            markers: entry_markers::find_from_entry(conn, entry_id).await?,
            custom_field_entries: custom_field_map,
            text_entries: text_entries::find_from_entry(conn, entry_id, is_private).await?,
        }))
    } else {
        Ok(None)
    }
}