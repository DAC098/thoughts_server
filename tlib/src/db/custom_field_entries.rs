use std::collections::{HashMap};

use tokio_postgres::{GenericClient};
use serde::{Serialize, Deserialize};
use chrono::serde::{ts_seconds};

use crate::db::error;

#[derive(Serialize, Deserialize)]
pub struct CustomFieldEntry {
    pub field: i32,
    pub value: CustomFieldEntryType,
    pub comment: Option<String>,
    pub entry: i32
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum CustomFieldEntryType {
    Integer {
        value: i32
    },
    IntegerRange {
        low: i32,
        high: i32
    },

    Float {
        value: f32
    },
    FloatRange {
        low: f32,
        high: f32
    },

    Time {
        //#[serde(with = "ts_seconds")]
        value: chrono::DateTime<chrono::Utc>
    },
    TimeRange {
        //#[serde(with = "ts_seconds")]
        low: chrono::DateTime<chrono::Utc>,

        //#[serde(with = "ts_seconds")]
        high: chrono::DateTime<chrono::Utc>
    },
}

async fn find_from_entry_query(
    conn: &impl GenericClient,
    entry: &i32
) -> error::Result<std::vec::Vec<tokio_postgres::Row>> {
    Ok(
        conn.query(
            "\
            select field, \
                   value, \
                   comment, \
                   entry \
            from custom_field_entries \
            where entry = $1",
            &[&entry]
        )
        .await?
    )
}

pub async fn find_from_entry(
    conn: &impl GenericClient,
    entry: &i32
) -> error::Result<Vec<CustomFieldEntry>> {
    Ok(
        find_from_entry_query(conn, entry)
        .await?
        .iter()
        .map(|row| CustomFieldEntry {
            field: row.get(0),
            value: serde_json::from_value(row.get(1)).unwrap(),
            comment: row.get(2),
            entry: row.get(3)
        })
        .collect()
    )
}

pub async fn find_from_entry_hashmap(
    conn: &impl GenericClient,
    entry: &i32
) -> error::Result<HashMap<i32, CustomFieldEntry>> {
    Ok(
        find_from_entry_query(conn, entry)
        .await?
        .iter()
        .fold(HashMap::new(), |mut map, row| {
            map.insert(row.get::<usize, i32>(0), CustomFieldEntry {
                field: row.get(0),
                value: serde_json::from_value(row.get(1)).unwrap(),
                comment: row.get(2),
                entry: row.get(3)
            });
            map
        })
    )
}