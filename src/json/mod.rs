use std::fmt::{Write};
use std::collections::{HashMap};
use std::marker::{Sync};

use tokio_postgres::{GenericClient, types::ToSql};
use serde::{Deserialize, Serialize};

use crate::db::custom_fields;
use crate::db::custom_field_entries;
use crate::error;

#[derive(Serialize, Clone)]
pub struct CustomFieldEntryJson {
    pub field: i32,
    pub name: String,
    pub value: custom_field_entries::CustomFieldEntryType,
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

#[derive(Serialize, Clone)]
pub struct EntryTagJson {
    pub id: i32,
    pub tag_id: i32,
    pub title: String,
    pub color: String,
    pub owner: i32,
    pub entry: i32
}

#[derive(Serialize)]
pub struct EntryJson {
    pub id: i32,
    pub created: chrono::DateTime<chrono::Utc>,
    pub owner: i32,
    pub tags: Vec<i32>,
    pub custom_field_entries: Vec<CustomFieldEntryJson>,
    pub text_entries: Vec<TextEntryJson>
}

#[derive(Serialize)]
pub struct IssuedByJson {
    pub id: i32,
    pub username: i32,
    pub full_name: Option<String>
}

#[derive(Serialize)]
pub struct CustomFieldJson {
    pub id: i32,
    pub name: String,
    pub comment: Option<String>,
    pub config: custom_fields::CustomFieldType,
    pub owner: i32,
    pub issued_by: Option<IssuedByJson>
}

#[derive(Serialize, Clone)]
pub struct TagJson {
    pub id: i32,
    pub title: String,
    pub owner: i32,
    pub color: String,
    pub comment: Option<String>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserAccessInfoJson {
    pub id: i32,
    pub username: String,
    pub full_name: Option<String>,
    pub ability: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserInfoJson {
    pub id: i32,
    pub username: String,
    pub level: i32,
    pub full_name: Option<String>,
    pub email: String,
    pub user_access: Vec<UserAccessInfoJson>
}

pub async fn search_custom_fields(
    conn: &impl GenericClient,
    owner: i32,
) -> error::Result<Vec<CustomFieldJson>> {
    let rows = conn.query(
        r#"
        select custom_fields.id as id,
               custom_fields.name as name, 
               custom_fields.config as config,
               custom_fields.comment as comment,
               custom_fields.owner as owner,
               custom_fields.issued_by as issued_by,
               users.username as username,
               users.full_name as full_name
        from custom_fields
        left join users on custom_fields.issued_by = users.id
        where owner = $1
        order by id asc
        "#,
        &[&owner]
    ).await?;
    let mut rtn = Vec::<CustomFieldJson>::with_capacity(rows.len());

    for row in rows {
        let issued_by = match row.get::<usize, Option<i32>>(5) {
            Some(id) => Some(IssuedByJson {
                id, username: row.get(6), full_name: row.get(7)
            }),
            None => None
        };

        rtn.push(CustomFieldJson {
            id: row.get(0),
            name: row.get(1),
            comment: row.get(3),
            config: serde_json::from_value(row.get(2)).unwrap(),
            owner: row.get(4),
            issued_by
        });
    }

    Ok(rtn)
}

pub async fn search_custom_field(
    conn: &impl GenericClient,
    field_id: i32
) -> error::Result<Option<CustomFieldJson>> {
    let rows = conn.query(
        r#"
        select custom_fields.id as id,
               custom_fields.name as name, 
               custom_fields.config as config,
               custom_fields.comment as comment,
               custom_fields.owner as owner,
               custom_fields.issued_by as issued_by,
               users.username as username,
               users.full_name as full_name
        from custom_fields
        left join users on custom_fields.issued_by = users.id
        where custom_fields.id = $1
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

        Ok(Some(CustomFieldJson {
            id: rows[0].get(0),
            name: rows[0].get(1),
            config: serde_json::from_value(rows[0].get(2)).unwrap(),
            comment: rows[0].get(3),
            owner: rows[0].get(4),
            issued_by
        }))
    } else {
        Ok(None)
    }
}

fn search_text_entries_query_slice<'a>(
    entry_ids: &'a Vec<i32>,
    is_private: &'a Option<bool>
) -> error::Result<(String, Vec<&'a (dyn ToSql + Sync)>)> {
    let arg_count: u32 = 2;
    let mut query_str = r#"
    select text_entries.id as id,
           text_entries.thought as thought,
           text_entries.entry as entry,
           text_entries.private as private
    from text_entries
    where "#.to_owned();
    let mut query_slice: Vec<&(dyn ToSql + Sync)> = vec!();

    if entry_ids.len() == 1 {
        write!(&mut query_str, "text_entries.entry = $1")?;
        query_slice.push(&entry_ids[0]);
    } else {
        write!(&mut query_str, "text_entries.entry = any($1)")?;
        query_slice.push(entry_ids);
    }

    if let Some(private) = is_private {
        write!(&mut query_str, " and text_entries.private = ${}", arg_count)?;
        query_slice.push(private);
    }

    write!(&mut query_str, "\n    order by text_entries.entry asc")?;

    Ok((query_str, query_slice))
}

fn search_custom_field_entries_query_slice<'a>(
    entry_ids: &'a Vec<i32>,
) -> error::Result<(String, Vec<&'a(dyn ToSql + Sync)>)> {
    let mut query_str = r#"
    select custom_fields.id as field,
           custom_fields.name as name,
           custom_field_entries.value as value,
           custom_field_entries.comment as comment,
           custom_field_entries.entry as entry
    from custom_field_entries
    join custom_fields on custom_field_entries.field = custom_fields.id
    where "#.to_owned();
    let mut query_slice: Vec<&(dyn ToSql + Sync)> = vec!();

    if entry_ids.len() == 1 {
        write!(&mut query_str, "custom_field_entries.entry = $1")?;
        query_slice.push(&entry_ids[0]);
    } else {
        write!(&mut query_str, "custom_field_entries.entry = any($1)")?;
        query_slice.push(entry_ids);
    }

    write!(&mut query_str, "    order by custom_field_entries.entry asc, custom_field_entries.field asc")?;

    Ok((query_str, query_slice))
}

fn search_tag_entries_query_slice<'a>(
    entry_ids: &'a Vec<i32>,
) -> error::Result<(String, Vec<&'a(dyn ToSql + Sync)>)> {
    let mut query_str = "select tag, entry from entries2tags where ".to_owned();
    let mut query_slice: Vec<&(dyn ToSql + Sync)> = Vec::with_capacity(1);

    if entry_ids.len() == 1 {
        write!(&mut query_str, "entry = $1")?;
        query_slice.push(&entry_ids[0]);
    } else {
        write!(&mut query_str, "entry = any($1)")?;
        query_slice.push(entry_ids);
    }

    Ok((query_str, query_slice))
}

pub async fn search_text_entries(
    conn: &impl GenericClient,
    entry_id: &i32,
    is_private: Option<bool>,
) -> error::Result<Vec<TextEntryJson>> {
    let entry_ids: Vec<i32> = vec!(*entry_id);
    let (query_str, query_slice) = search_text_entries_query_slice(&entry_ids, &is_private)?;

    Ok(conn.query(query_str.as_str(), &query_slice[..])
        .await?
        .iter()
        .map(|row| TextEntryJson{
            id: row.get(0),
            thought: row.get(1),
            entry: row.get(2),
            private: row.get(3)
        })
        .collect())
}

pub async fn search_custom_field_entries(
    conn: &impl GenericClient,
    entry_id: &i32,
) -> error::Result<Vec<CustomFieldEntryJson>> {
    let entry_ids: Vec<i32> = vec!(*entry_id);
    let (query_str, query_slice) = search_custom_field_entries_query_slice(&entry_ids)?;

    Ok(conn.query(query_str.as_str(), &query_slice[..])
        .await?
        .iter()
        .map(|row| CustomFieldEntryJson {
            field: row.get(0),
            name: row.get(1),
            value: serde_json::from_value(row.get(2)).unwrap(),
            comment: row.get(3),
            entry: row.get(4)
        })
        .collect())
}

pub async fn search_tag_entries(
    conn: &impl GenericClient,
    entry_id: &i32,
) -> error::Result<Vec<i32>> {
    let entry_ids: Vec<i32> = vec!(*entry_id);
    let (query_str, query_slice) = search_tag_entries_query_slice(&entry_ids)?;

    Ok(conn.query(query_str.as_str(), &query_slice[..])
        .await?
        .iter()
        .map(|row| row.get::<usize, i32>(0))
        .collect())
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
    let rows = {
        let mut arg_count: u32 = 2;
        let mut query_str = "select id, day, owner from entries where owner = $1".to_owned();
        let mut query_slice: Vec<&(dyn ToSql + Sync)> = vec!(&options.owner);

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

        conn.query(query_str.as_str(), &query_slice[..]).await?
    };
    let mut entry_ids: Vec<i32> = Vec::with_capacity(rows.len());
    let mut entry_hash_map: HashMap<i32, usize> = HashMap::with_capacity(rows.len());
    let mut rtn: Vec<EntryJson> = Vec::with_capacity(rows.len());
    let mut count: usize = 0;

    if rows.len() == 0 {
        return Ok(rtn);
    }

    for row in rows {
        let entry_id = row.get(0);
        entry_ids.push(entry_id);
        rtn.push(EntryJson {
            id: entry_id,
            created: row.get(1),
            owner: row.get(2),
            tags: vec!(),
            custom_field_entries: vec!(),
            text_entries: vec!()
        });
        entry_hash_map.insert(entry_id, count);
        count += 1;
    }

    {
        let custom_field_entries = {
            let (query_str, query_slice) = search_custom_field_entries_query_slice(&entry_ids)?;

            conn.query(query_str.as_str(), &query_slice[..]).await?
        };
        let mut current_set: Vec<CustomFieldEntryJson> = vec!();
        let mut current_entry_id: i32 = 0;

        for row in custom_field_entries {
            let entry_id: i32 = row.get(5);

            if current_entry_id == 0 {
                current_entry_id = entry_id;
            } else if current_entry_id != entry_id {
                let borrow = entry_hash_map.get(&current_entry_id).unwrap();
                rtn[*borrow].custom_field_entries.reserve(current_set.len());
                rtn[*borrow].custom_field_entries.append(&mut current_set);
                current_entry_id = entry_id;
            }

            current_set.push(CustomFieldEntryJson {
                field: row.get(0),
                name: row.get(1),
                value: serde_json::from_value(row.get(2)).unwrap(),
                comment: row.get(3),
                entry: entry_id
            });
        }

        if entry_ids.len() > 0 && current_entry_id != 0 {
            let borrow = entry_hash_map.get(&current_entry_id).unwrap();
            rtn[*borrow].custom_field_entries.reserve(current_set.len());
            rtn[*borrow].custom_field_entries.append(&mut current_set);
        }
    }

    {
        let text_entries = {
            let (query_str, query_slice) = search_text_entries_query_slice(&entry_ids, &options.is_private)?;

            conn.query(query_str.as_str(), &query_slice[..]).await?
        };
        let mut current_set: Vec<TextEntryJson> = vec!();
        let mut current_entry_id: i32 = 0;

        for row in text_entries {
            let entry_id: i32 = row.get(2);

            if current_entry_id == 0 {
                current_entry_id = entry_id;
            } else if current_entry_id != entry_id {
                let borrow = entry_hash_map.get(&current_entry_id).unwrap();
                rtn[*borrow].text_entries.reserve(current_set.len());
                rtn[*borrow].text_entries.append(&mut current_set);
                current_entry_id = entry_id;
            }

            current_set.push(TextEntryJson {
                id: row.get(0),
                thought: row.get(1),
                entry: row.get(2),
                private: row.get(3)
            });
        }

        if entry_ids.len() > 0 && current_entry_id != 0 {
            let borrow = entry_hash_map.get(&current_entry_id).unwrap();
            rtn[*borrow].text_entries.reserve(current_set.len());
            rtn[*borrow].text_entries.append(&mut current_set);
        }
    }

    {
        let entries_tags = {
            let (query_str, query_slice) = search_tag_entries_query_slice(&entry_ids)?;

            conn.query(query_str.as_str(), &query_slice[..]).await?
        };
        let mut current_set: Vec<i32> = vec!();
        let mut current_entry_id: i32 = 0;

        for row in entries_tags {
            let entry_id: i32 = row.get(1);

            if current_entry_id == 0 {
                current_entry_id = entry_id;
            } else if entry_id != current_entry_id {
                let borrow = entry_hash_map.get(&current_entry_id).unwrap();
                rtn[*borrow].tags.reserve(current_set.len());
                rtn[*borrow].tags.append(&mut current_set);
                current_entry_id = entry_id;
            }

            current_set.push(row.get(0));
        }

        if entry_ids.len() > 0 && current_entry_id != 0 {
            let borrow = entry_hash_map.get(&current_entry_id).unwrap();
            rtn[*borrow].tags.reserve(current_set.len());
            rtn[*borrow].tags.append(&mut current_set);
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
        let tags = conn.query("select tag from entries2tags where entry = $1", &[&entry_id])
            .await?
            .iter()
            .map(|row| row.get::<usize, i32>(0))
            .collect();

        Ok(Some(EntryJson {
            id: entry_id,
            created: rows[0].get(1),
            owner: rows[0].get(2),
            tags,
            custom_field_entries: search_custom_field_entries(conn, &entry_id).await?,
            text_entries: search_text_entries(conn, &entry_id, is_private).await?
        }))
    } else {
        Ok(None)
    }
}