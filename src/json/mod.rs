use std::fmt::Write;
use std::collections::HashMap;

use tokio_postgres::GenericClient;

use crate::db::{
    custom_field_entries,
    entries,
    entry_markers,
    text_entries,
    composed,
};
use crate::db::query::QueryParams;

use crate::response::error;

fn search_text_entries_query_slice<'a>(
    entry_ids: &'a Vec<i32>,
    is_private: &'a Option<bool>
) -> error::Result<(String, QueryParams<'a>)> {
    let mut query_str = "\
    select text_entries.id as id, \
           text_entries.thought as thought, \
           text_entries.entry as entry, \
           text_entries.private as private \
    from text_entries \
    where ".to_owned();
    let mut query_slice: QueryParams<'a> = QueryParams::new();

    if entry_ids.len() == 1 {
        write!(&mut query_str, "text_entries.entry = $1")?;
        query_slice.push(&entry_ids[0]);
    } else {
        write!(&mut query_str, "text_entries.entry = any($1)")?;
        query_slice.push(entry_ids);
    }

    if let Some(private) = is_private {
        write!(&mut query_str, " and text_entries.private = ${}", query_slice.next())?;
        query_slice.push(private);
    }

    write!(&mut query_str, " order by text_entries.entry asc")?;

    Ok((query_str, query_slice))
}

fn search_custom_field_entries_query_slice<'a>(
    entry_ids: &'a Vec<i32>,
) -> error::Result<(String, QueryParams<'a>)> {
    let mut query_str = "\
    select custom_fields.id as field, \
           custom_fields.name as name, \
           custom_field_entries.value as value, \
           custom_field_entries.comment as comment, \
           custom_field_entries.entry as entry \
    from custom_field_entries \
    join custom_fields on custom_field_entries.field = custom_fields.id \
    where ".to_owned();
    let mut query_slice: QueryParams = QueryParams::with_capacity(1);

    if entry_ids.len() == 1 {
        write!(&mut query_str, "custom_field_entries.entry = $1")?;
        query_slice.push(&entry_ids[0]);
    } else {
        write!(&mut query_str, "custom_field_entries.entry = any($1)")?;
        query_slice.push(entry_ids);
    }

    write!(&mut query_str, " order by custom_field_entries.entry asc, custom_fields.\"order\", custom_fields.name")?;

    Ok((query_str, query_slice))
}

fn search_tag_entries_query_slice<'a>(
    entry_ids: &'a Vec<i32>,
) -> error::Result<(String, QueryParams<'a>)> {
    let mut query_str = "select tag, entry from entries2tags where ".to_owned();
    let mut query_slice: QueryParams = QueryParams::with_capacity(1);

    if entry_ids.len() == 1 {
        write!(&mut query_str, "entry = $1")?;
        query_slice.push(&entry_ids[0]);
    } else {
        write!(&mut query_str, "entry = any($1)")?;
        query_slice.push(entry_ids);
    }

    Ok((query_str, query_slice))
}

fn search_entry_markers_query_slice<'a>(
    entry_ids: &'a Vec<i32>,
) -> error::Result<(String, QueryParams<'a>)> {
    let mut query_str = "select id, title, comment, entry from entry_markers where ".to_owned();
    let mut query_params: QueryParams = QueryParams::with_capacity(1);

    if entry_ids.len() == 1 {
        write!(&mut query_str, "entry = $1")?;
        query_params.push(&entry_ids[0]);
    } else {
        write!(&mut query_str, "entry = any($1)")?;
        query_params.push(entry_ids);
    }

    Ok((query_str, query_params))
}

pub struct SearchEntriesOptions {
    pub owner: i32,
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    pub to: Option<chrono::DateTime<chrono::Utc>>,
    pub tags: Option<Vec<i32>>,
    pub is_private: Option<bool>
}

pub async fn search_entries(
    conn: &impl GenericClient, 
    options: SearchEntriesOptions
) -> error::Result<Vec<composed::ComposedEntry>> {
    let rows = {
        let mut query_str = "select id, day, owner from entries where owner = $1".to_owned();
        let mut query_slice: QueryParams = QueryParams::with_capacity(1);
        query_slice.push(&options.owner);

        if let Some(from) = options.from.as_ref() {
            write!(&mut query_str, " and day >= ${}", query_slice.push(from))?;
        }

        if let Some(to) = options.to.as_ref() {
            write!(&mut query_str, " and day <= ${}", query_slice.push(to))?;
        }

        if let Some(tags) = options.tags.as_ref() {
            write!(&mut query_str, " and id in (select entry from entries2tags where tag = any(${}))", query_slice.push(tags))?;
        }

        write!(&mut query_str, " order by day desc")?;

        conn.query(query_str.as_str(), query_slice.slice()).await?
    };
    let mut entry_ids: Vec<i32> = Vec::with_capacity(rows.len());
    let mut entry_hash_map: HashMap<i32, usize> = HashMap::with_capacity(rows.len());
    let mut rtn: Vec<composed::ComposedEntry> = Vec::with_capacity(rows.len());
    let mut count: usize = 0;

    if rows.len() == 0 {
        return Ok(rtn);
    }

    for row in rows {
        let entry_id = row.get(0);
        entry_ids.push(entry_id);
        rtn.push(composed::ComposedEntry {
            entry: entries::Entry {
                id: entry_id,
                day: row.get(1),
                owner: row.get(2),
            },
            tags: vec!(),
            markers: vec!(),
            custom_field_entries: HashMap::new(),
            text_entries: vec!()
        });
        entry_hash_map.insert(entry_id, count);
        count += 1;
    }

    {
        let custom_field_entries = {
            let (query_str, query_slice) = search_custom_field_entries_query_slice(&entry_ids)?;

            conn.query(query_str.as_str(), query_slice.slice()).await?
        };
        let mut current_set: HashMap<i32, custom_field_entries::CustomFieldEntry> = HashMap::new();
        let mut current_entry_id: i32 = 0;

        for row in custom_field_entries {
            let entry_id: i32 = row.get(4);

            if current_entry_id == 0 {
                current_entry_id = entry_id;
            } else if current_entry_id != entry_id {
                let borrow = entry_hash_map.get(&current_entry_id).unwrap();
                std::mem::swap(&mut rtn[*borrow].custom_field_entries, &mut current_set);
                current_entry_id = entry_id;
            }

            current_set.insert(row.get(0), custom_field_entries::CustomFieldEntry {
                field: row.get(0),
                value: serde_json::from_value(row.get(2)).unwrap(),
                comment: row.get(3),
                entry: entry_id
            });
        }

        if entry_ids.len() > 0 && current_entry_id != 0 {
            let borrow = entry_hash_map.get(&current_entry_id).unwrap();
            std::mem::swap(&mut rtn[*borrow].custom_field_entries, &mut current_set);
        }
    }

    {
        let text_entries = {
            let (query_str, query_slice) = search_text_entries_query_slice(&entry_ids, &options.is_private)?;

            conn.query(query_str.as_str(), query_slice.slice()).await?
        };
        let mut current_set: Vec<text_entries::TextEntry> = vec!();
        let mut current_entry_id: i32 = 0;

        for row in text_entries {
            let entry_id: i32 = row.get(2);

            if current_entry_id == 0 {
                current_entry_id = entry_id;
            } else if current_entry_id != entry_id {
                let borrow = entry_hash_map.get(&current_entry_id).unwrap();
                std::mem::swap(&mut rtn[*borrow].text_entries, &mut current_set);
                current_entry_id = entry_id;
            }

            current_set.push(text_entries::TextEntry {
                id: row.get(0),
                thought: row.get(1),
                entry: row.get(2),
                private: row.get(3)
            });
        }

        if entry_ids.len() > 0 && current_entry_id != 0 {
            let borrow = entry_hash_map.get(&current_entry_id).unwrap();
            std::mem::swap(&mut rtn[*borrow].text_entries, &mut current_set);
        }
    }

    {
        let entries_tags = {
            let (query_str, query_slice) = search_tag_entries_query_slice(&entry_ids)?;

            conn.query(query_str.as_str(), query_slice.slice()).await?
        };
        let mut current_set: Vec<i32> = vec!();
        let mut current_entry_id: i32 = 0;

        for row in entries_tags {
            let entry_id: i32 = row.get(1);

            if current_entry_id == 0 {
                current_entry_id = entry_id;
            } else if entry_id != current_entry_id {
                let borrow = entry_hash_map.get(&current_entry_id).unwrap();
                std::mem::swap(&mut rtn[*borrow].tags, &mut current_set);
                current_entry_id = entry_id;
            }

            current_set.push(row.get(0));
        }

        if entry_ids.len() > 0 && current_entry_id != 0 {
            let borrow = entry_hash_map.get(&current_entry_id).unwrap();
            std::mem::swap(&mut rtn[*borrow].tags, &mut current_set);
        }
    }

    {
        let entry_markers = {
            let (query_str, query_slice) = search_entry_markers_query_slice(&entry_ids)?;

            conn.query(query_str.as_str(), query_slice.slice()).await?
        };
        let mut current_set: Vec<entry_markers::EntryMarker> = vec!();
        let mut current_entry_id: i32 = 0;

        for row in entry_markers {
            let entry_id: i32 = row.get(3);

            if current_entry_id == 0 {
                current_entry_id = entry_id;
            } else if entry_id != current_entry_id {
                let borrow = entry_hash_map.get(&current_entry_id).unwrap();
                std::mem::swap(&mut rtn[*borrow].markers, &mut current_set);
                current_entry_id = entry_id;
            }

            current_set.push(entry_markers::EntryMarker {
                id: row.get(0),
                title: row.get(1),
                comment: row.get(2),
                entry: row.get(3)
            });
        }

        if entry_ids.len() > 0 && current_entry_id != 0 {
            let borrow = entry_hash_map.get(&current_entry_id).unwrap();
            std::mem::swap(&mut rtn[*borrow].markers, &mut current_set);
        }
    }

    Ok(rtn)
}