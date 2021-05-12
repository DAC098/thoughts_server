use std::fmt::{Write};
use std::collections::{HashMap};

use tokio_postgres::{GenericClient};
use serde::{Deserialize, Serialize};

use crate::db::mood_fields;
use crate::db::mood_entries;
use crate::error;

#[derive(Serialize, Clone)]
pub struct MoodEntryJson {
    pub id: i32,
    pub field: String,
    pub field_id: i32,
    pub value: mood_entries::MoodEntryType,
    pub comment: Option<String>,
    pub entry: i32
}

#[derive(Serialize, Clone)]
pub struct TextEntryJson {
    pub id: i32,
    pub thought: String,
    pub private: bool,
    pub entry: i32
}

#[derive(Serialize)]
pub struct EntryJson {
    pub id: i32,
    pub created: chrono::DateTime<chrono::Utc>,
    pub owner: i32,
    pub mood_entries: Vec<MoodEntryJson>,
    pub text_entries: Vec<TextEntryJson>
}

#[derive(Serialize)]
pub struct IssuedByJson {
    pub id: i32,
    pub username: i32,
    pub full_name: Option<String>
}

#[derive(Serialize)]
pub struct MoodFieldJson {
    pub id: i32,
    pub name: String,
    pub comment: Option<String>,
    pub config: mood_fields::MoodFieldType,
    pub owner: i32,
    pub issued_by: Option<IssuedByJson>
}

pub async fn search_mood_fields(
    conn: &impl GenericClient,
    owner: i32,
) -> error::Result<Vec<MoodFieldJson>> {
    let rows = conn.query(
        r#"
        select mood_fields.id as id,
               mood_fields.name as name, 
               mood_fields.config as config,
               mood_fields.comment as comment,
               mood_fields.owner as owner,
               mood_fields.issued_by as issued_by,
               users.username as username,
               users.full_name as full_name
        from mood_fields
        left join users on mood_fields.issued_by = users.id
        where owner = $1
        order by id asc
        "#,
        &[&owner]
    ).await?;
    let mut rtn = Vec::<MoodFieldJson>::with_capacity(rows.len());

    for row in rows {
        let issued_by = match row.get::<usize, Option<i32>>(5) {
            Some(id) => Some(IssuedByJson {
                id, username: row.get(6), full_name: row.get(7)
            }),
            None => None
        };

        rtn.push(MoodFieldJson {
            id: row.get(0),
            name: row.get(1),
            comment: row.get(3),
            config: serde_json::from_value(row.get(2))?,
            owner: row.get(4),
            issued_by
        });
    }

    Ok(rtn)
}

pub async fn search_mood_field(
    conn: &impl GenericClient,
    field_id: i32
) -> error::Result<Option<MoodFieldJson>> {
    let rows = conn.query(
        r#"
        select mood_fields.id as id,
               mood_fields.name as name, 
               mood_fields.config as config,
               mood_fields.comment as comment,
               mood_fields.owner as owner,
               mood_fields.issued_by as issued_by,
               users.username as username,
               users.full_name as full_name
        from mood_fields
        left join users on mood_fields.issued_by = users.id
        where mood_fields.id = $1
        "#,
        &[&field_id]
    ).await?;

    if rows.len() == 1 {
        let issued_by = match rows[0].get::<usize, Option<i32>>(5) {
            Some(id) => Some(IssuedByJson {
                id, username: rows[0].get(6), full_name: rows[0].get(7)
            }),
            None => None
        };

        Ok(Some(MoodFieldJson {
            id: rows[0].get(0),
            name: rows[0].get(1),
            config: serde_json::from_value(rows[0].get(2))?,
            comment: rows[0].get(3),
            owner: rows[0].get(4),
            issued_by
        }))
    } else {
        Ok(None)
    }
}

pub async fn search_text_entries(
    conn: &impl GenericClient,
    entry_ids: &Vec<i32>,
    is_private: Option<bool>,
) -> error::Result<Vec<TextEntryJson>> {
    let arg_count: u32 = 2;
    let mut query_str = r#"
    select text_entries.id as id,
           text_entries.thought as thought,
           text_entries.entry as entry,
           text_entries.private as private
    from text_entries
    where text_entries.entry = any($1)
    "#.to_owned();
    let mut query_slice: Vec<&(dyn tokio_postgres::types::ToSql + std::marker::Sync)> = vec![&entry_ids];

    if let Some(private) = is_private.as_ref() {
        write!(&mut query_str, " and text_entries.private = ${}", arg_count)?;
        query_slice.push(private);
    }

    write!(&mut query_str, "\n    order by text_entries.entry asc")?;

    let rows = conn.query(query_str.as_str(), &query_slice[..]).await?;
    let mut rtn = Vec::<TextEntryJson>::with_capacity(rows.len());

    for row in rows {
        rtn.push(TextEntryJson{
            id: row.get(0),
            thought: row.get(1),
            entry: row.get(2),
            private: row.get(3)
        });
    }

    Ok(rtn)
}

pub async fn search_mood_entries(
    conn: &impl GenericClient,
    entry_ids: &Vec<i32>,
) -> error::Result<Vec<MoodEntryJson>> {
    let rows = conn.query(
        r#"
        select mood_entries.id as id,
               mood_fields.name as field,
               mood_fields.id as field_id,
               mood_entries.value as value,
               mood_entries.comment as comment,
               mood_entries.entry as entry
        from mood_entries
        join mood_fields on mood_entries.field = mood_fields.id
        where mood_entries.entry = any($1)
        order by mood_entries.entry asc,
                 mood_entries.field asc
        "#, 
        &[entry_ids]
    ).await?;
    let mut rtn = Vec::<MoodEntryJson>::with_capacity(rows.len());

    for row in rows {
        rtn.push(MoodEntryJson {
            id: row.get(0),
            field: row.get(1),
            field_id: row.get(2),
            value: serde_json::from_value(row.get(3))?,
            comment: row.get(4),
            entry: row.get(5)
        });
    }

    Ok(rtn)
}

#[derive(Deserialize)]
pub struct QueryEntries {
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    pub to: Option<chrono::DateTime<chrono::Utc>>
}

pub struct SearchEntriesOptions {
    pub owner: i32,
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    pub to: Option<chrono::DateTime<chrono::Utc>>,
    pub is_private: Option<bool>
}

pub async fn search_entries(
    conn: &impl GenericClient, 
    options: SearchEntriesOptions
) -> error::Result<Vec<EntryJson>> {
    let mut arg_count: u32 = 2;
    let mut query_str = "select id, day, owner from entries where owner = $1".to_owned();
    let mut query_slice: Vec<&(dyn tokio_postgres::types::ToSql + std::marker::Sync)> = vec!(&options.owner);

    if let Some(from) = options.from.as_ref() {
        write!(&mut query_str, " and day >= ${}", arg_count)?;
        query_slice.push(from);
        arg_count += 1;
    }

    if let Some(to) = options.to.as_ref() {
        write!(&mut query_str, " and day <= ${}", arg_count)?;
        query_slice.push(to);
    }

    write!(&mut query_str, " order by day desc")?;

    let rows = conn.query(query_str.as_str(), &query_slice[..]).await?;
    let mut entry_ids = Vec::<i32>::with_capacity(rows.len());
    let mut entry_hash_map = HashMap::<i32, usize>::with_capacity(rows.len());
    let mut rtn = Vec::<EntryJson>::with_capacity(rows.len());
    let mut count: usize = 0;

    if rows.len() == 0 {
        return Ok(rtn);
    }

    for row in rows {
        let entry_id = row.get(0);
        entry_ids.push(entry_id);
        &rtn.push(EntryJson {
            id: entry_id,
            created: row.get(1),
            owner: row.get(2),
            mood_entries: vec!(),
            text_entries: vec!()
        });
        entry_hash_map.insert(entry_id, count);
        count += 1;
    }

    {
        let mut mood_entries = search_mood_entries(conn, &entry_ids).await?;
        let mut current_set: Vec<MoodEntryJson> = vec!();
        let mut current_entry_id = if mood_entries.len() > 0 { 
            mood_entries[mood_entries.len() - 1].entry
        } else { 0 };

        while let Some(mood) = mood_entries.pop() {
            if mood.entry != current_entry_id {
                let borrow = entry_hash_map.get(&current_entry_id).unwrap();
                rtn[*borrow].mood_entries.reserve(current_set.len());
                rtn[*borrow].mood_entries.append(&mut current_set);
                current_entry_id = mood.entry;
            }

            current_set.push(mood);
        }

        if entry_ids.len() > 0 && current_entry_id != 0 {
            let borrow = entry_hash_map.get(&current_entry_id).unwrap();
            rtn[*borrow].mood_entries.reserve(current_set.len());
            rtn[*borrow].mood_entries.append(&mut current_set);
        }
    }

    {
        let mut text_entries = search_text_entries(conn, &entry_ids, options.is_private).await?;
        let mut current_set: Vec<TextEntryJson> = vec!();
        let mut current_entry_id = if text_entries.len() > 0 { 
            text_entries[text_entries.len() - 1].entry 
        } else { 0 };

        while let Some(text) = text_entries.pop() {
            if text.entry != current_entry_id {
                let borrow = entry_hash_map.get(&current_entry_id).unwrap();
                rtn[*borrow].text_entries.reserve(current_set.len());
                rtn[*borrow].text_entries.append(&mut current_set);
                current_entry_id = text.entry;
            }

            &current_set.push(text);
        }

        if entry_ids.len() > 0 && current_entry_id != 0 {
            let borrow = entry_hash_map.get(&current_entry_id).unwrap();
            rtn[*borrow].text_entries.reserve(current_set.len());
            rtn[*borrow].text_entries.append(&mut current_set);
        }
    }

    Ok(rtn)
}

pub async fn search_entry(
    conn: &impl GenericClient,
    entry_id: i32,
    is_private: Option<bool>,
) -> error::Result<Option<EntryJson>> {
    let rows = conn.query(
        "select id, day, owner from entries where id = $1",
        &[&entry_id]
    ).await?;

    if rows.len() != 0 {
        let entry_ids: Vec<i32> = vec!(entry_id);

        Ok(Some(EntryJson {
            id: entry_id,
            created: rows[0].get(1),
            owner: rows[0].get(2),
            mood_entries: search_mood_entries(conn, &entry_ids).await?,
            text_entries: search_text_entries(conn, &entry_ids, is_private).await?
        }))
    } else {
        Ok(None)
    }
}