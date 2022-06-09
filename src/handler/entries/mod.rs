use std::fmt::Write;
use std::collections::HashMap;
//use std::pin::{Pin};
//use std::task::{Context, Poll};

use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;
use chrono::{DateTime, Utc};
use chrono::serde::ts_seconds;
//use tokio_postgres::{Client, RowStream};
//use futures::{pin_mut, Stream, TryStreamExt, future};
use futures::future;

use crate::db::{
    self,
    custom_field_entries,
    entries,
    entry_markers,
    text_entries,
    composed,
};

use db::composed::ComposedEntry;

pub mod entry_id;

use crate::response;
use crate::response::json::JsonBuilder;
use crate::state;
use crate::request::{initiator_from_request, Initiator};
use crate::getters;
use crate::security;
use crate::parsing;

use response::error as app_error;

#[derive(Deserialize)]
pub struct PostTextEntryJson {
    thought: String,
    private: bool
}

#[derive(Deserialize)]
pub struct PostCustomFieldEntryJson {
    field: i32,
    value: db::custom_field_entries::CustomFieldEntryType,
    comment: Option<String>
}

#[derive(Deserialize)]
pub struct PostEntryMarker {
    title: String,
    comment: Option<String>
}

#[derive(Deserialize)]
pub struct PostEntry {
    #[serde(with = "ts_seconds")]
    day: chrono::DateTime<chrono::Utc>
}

#[derive(Deserialize)]
pub struct PostEntryJson {
    entry: PostEntry,
    tags: Option<Vec<i32>>,
    custom_field_entries: Option<Vec<PostCustomFieldEntryJson>>,
    text_entries: Option<Vec<PostTextEntryJson>>,
    markers: Option<Vec<PostEntryMarker>>
}

#[derive(Deserialize)]
pub struct EntriesPath {
    user_id: Option<i32>
}

#[derive(Deserialize)]
pub struct EntriesQuery {
    from: Option<String>,
    to: Option<String>,
    tags: Option<String>,
    from_marker: Option<i32>,
    to_marker: Option<i32>,
}

/**
 * GET /entries
 * returns the root html if requesting html. otherwise will send back a list of
 * available and allowed entries for the current user from the session
 */
pub async fn handle_get(
    req: HttpRequest,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
    info: web::Query<EntriesQuery>,
    path: web::Path<EntriesPath>,
) -> app_error::Result<impl Responder> {
    let info = info.into_inner();
    let pool_conn = db.pool.get().await?;
    let accept_html = response::try_check_if_html_req(&req);
    let initiator_opt = initiator_from_request(&*pool_conn, &req).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(initiator_opt.unwrap().user))?)
        } else {
            Ok(response::redirect_to_login(&req))
        }
    } else if initiator_opt.is_none() {
        Err(app_error::ResponseError::Session)
    } else {
        let owner: i32;
        let is_private: Option<bool>;
        let initiator = initiator_opt.unwrap();

        if let Some(user_id) = path.user_id {
            security::assert::permission_to_read(&*pool_conn, &initiator.user.id, &user_id).await?;
            is_private = Some(false);
            owner = user_id;
        } else {
            is_private = None;
            owner = initiator.user.id;
        }

        let mut results = {
            let mut entries_where: String = String::new();

            write!(&mut entries_where, "entries.owner = {}", owner)?;

            if let Some(from_marker) = info.from_marker {
                let marker_check = (*pool_conn).query(
                    "\
                    select entries.day \
                    from entries \
                    join entry_markers on entries.id = entry_markers.entry \
                    where entry_markers.id = $1 and entries.owner = $2",
                    &[&from_marker, &owner]
                ).await?;

                if marker_check.is_empty() {
                    Err(app_error::ResponseError::BadRequest(
                        format!("from maker id given does not exist: {}", from_marker)
                    ))?;
                } else {
                    let day: DateTime<Utc> = marker_check[0].get(0);
                    write!(&mut entries_where, " and entries.day >= '{}'", day.to_rfc3339())?;
                }
            } else if let Some(from) = parsing::url_query::get_date(&info.from)? {
                write!(&mut entries_where, " and entries.day >= '{}'", from.to_rfc3339())?;
            }

            if let Some(to_marker) = info.to_marker {
                let marker_check = (*pool_conn).query(
                    "\
                    select entries.day \
                    from entries \
                    join entry_markers on entries.id = entry_markers.entry \
                    where entry_markers.id = $1 and entries.owner = $2",
                    &[&to_marker, &owner]
                ).await?;

                if marker_check.is_empty() {
                    Err(app_error::ResponseError::BadRequest(
                        format!("to marker id given does not exist: {}", to_marker)
                    ))?;
                } else {
                    let day: DateTime<Utc> = marker_check[0].get(0);
                    write!(&mut entries_where, " and entries.day <= '{}'", day.to_rfc3339())?;
                }
            } else if let Some(to) = parsing::url_query::get_date(&info.to)? {
                write!(&mut entries_where, " and entries.day <= '{}'", to.to_rfc3339())?;
            }

            if let Some(tags) = parsing::url_query::get_tags(&info.tags) {
                write!(
                    &mut entries_where,
                    " and entries.id in (select entry from entries2tags where tag in ({}))", 
                    tags.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(",")
                )?;
            }

            let rows_statement = format!(
                "\
                select id, day, owner \
                from entries \
                where {} \
                order by day desc",
                entries_where
            );

            let custom_field_entries_statement = format!(
                "\
                select custom_field_entries.field, \
                       custom_field_entries.value, \
                       custom_field_entries.comment, \
                       custom_field_entries.entry \
                from custom_field_entries \
                join entries on custom_field_entries.entry = entries.id \
                join custom_fields on custom_field_entries.field = custom_fields.id \
                where {} \
                order by entries.day desc, custom_fields.\"order\"",
                entries_where
            );

            let entry_markers_statement = format!(
                "\
                select entry_markers.id, \
                       entry_markers.title, \
                       entry_markers.comment, \
                       entry_markers.entry \
                from entry_markers \
                join entries on entry_markers.entry = entries.id \
                where {} \
                order by entries.day desc, entry_markers.id",
                entries_where
            );

            let text_entries_statement = {
                let mut query_str = format!(
                    "\
                    select text_entries.id, \
                           text_entries.thought, \
                           text_entries.private, \
                           text_entries.entry \
                    from text_entries \
                    join entries on text_entries.entry = entries.id \
                    where {}",
                    entries_where
                );

                if let Some(is_private) = is_private {
                    write!(&mut query_str, " and text_entries.private = {}", if is_private { "true" } else { "false" })?;
                }

                write!(&mut query_str, " order by entries.day desc, text_entries.id")?;

                query_str
            };

            let tags_statement = format!(
                "\
                select entries2tags.tag, \
                       entries2tags.entry \
                from entries2tags \
                join entries on entries2tags.entry = entries.id \
                where {} \
                order by entries.day desc",
                entries_where
            );

            future::try_join_all(vec![
                (*pool_conn).query(rows_statement.as_str(), &[]),
                (*pool_conn).query(custom_field_entries_statement.as_str(), &[]),
                (*pool_conn).query(entry_markers_statement.as_str(), &[]),
                (*pool_conn).query(text_entries_statement.as_str(), &[]),
                (*pool_conn).query(tags_statement.as_str(), &[])
            ]).await?
        };

        let tags = results.pop().unwrap();
        let mut tags_iter = tags.iter().map(|row| (row.get::<usize, i32>(0), row.get::<usize, i32>(1)));
        let text_entries = results.pop().unwrap();
        let mut text_entries_iter = text_entries.iter().map(|row| db::text_entries::TextEntry {
            id: row.get(0),
            thought: row.get(1),
            private: row.get(2),
            entry: row.get(3)
        });
        let entry_markers = results.pop().unwrap();
        let mut entry_markers_iter = entry_markers.iter().map(|row| db::entry_markers::EntryMarker {
            id: row.get(0),
            title: row.get(1),
            comment: row.get(2),
            entry: row.get(3)
        });
        let custom_field_entries = results.pop().unwrap();
        let mut custom_field_entries_iter = custom_field_entries.iter().map(|row| db::custom_field_entries::CustomFieldEntry {
            field: row.get(0),
            value: serde_json::from_value(row.get(1)).unwrap(),
            comment: row.get(2),
            entry: row.get(3)
        });
        let rows = results.pop().unwrap();
        let rows_iter = rows.iter().map(|row| db::entries::Entry {
            id: row.get(0),
            day: row.get(1),
            owner: row.get(2)
        });

        let mut rtn: Vec<ComposedEntry> = Vec::with_capacity(rows.len());

        let mut fields_done = false;
        let mut markers_done = false;
        let mut text_done = false;
        let mut tags_done = false;
        let mut next_custom_field_entry: Option<db::custom_field_entries::CustomFieldEntry> = None;
        let mut next_entry_marker: Option<db::entry_markers::EntryMarker> = None;
        let mut next_text_entry: Option<db::text_entries::TextEntry> = None;
        let mut next_tag: Option<(i32, i32)> = None;

        for row in rows_iter {
            let mut custom_field_entries_vec: HashMap<i32, db::custom_field_entries::CustomFieldEntry> = HashMap::new();
            let mut entry_markers_vec: Vec<db::entry_markers::EntryMarker> = Vec::new();
            let mut text_entries_vec: Vec<db::text_entries::TextEntry> = Vec::new();
            let mut tags_vec: Vec<i32> = Vec::new();

            if let Some (refer) = next_custom_field_entry.as_ref() {
                if refer.entry == row.id {
                    custom_field_entries_vec.insert(refer.field, next_custom_field_entry.take().unwrap());
                }
            }

            if !fields_done && next_custom_field_entry.is_none() {
                loop {
                    if let Some(field) = custom_field_entries_iter.next() {
                        if field.entry == row.id {
                            custom_field_entries_vec.insert(field.field, field);
                        } else {
                            next_custom_field_entry = Some(field);
                            break;
                        }
                    } else {
                        fields_done = true;
                        break;
                    }
                }
            }

            if let Some(refer) = next_entry_marker.as_ref() {
                if refer.entry == row.id {
                    entry_markers_vec.push(next_entry_marker.take().unwrap());
                }
            }

            if !markers_done && next_entry_marker.is_none() {
                loop {
                    if let Some(marker) = entry_markers_iter.next() {
                        if marker.entry == row.id {
                            entry_markers_vec.push(marker);
                        } else {
                            next_entry_marker = Some(marker);
                            break;
                        }
                    } else {
                        markers_done = true;
                        break;
                    }
                }
            }

            if let Some(refer) = next_text_entry.as_ref() {
                if refer.entry == row.id {
                    text_entries_vec.push(next_text_entry.take().unwrap());
                }
            }

            if !text_done && next_text_entry.is_none() {
                loop {
                    if let Some(text) = text_entries_iter.next() {
                        if text.entry == row.id {
                            text_entries_vec.push(text);
                        } else {
                            next_text_entry = Some(text);
                            break;
                        }
                    } else {
                        text_done = true;
                        break;
                    }
                }
            }

            if let Some(refer) = next_tag.as_ref() {
                if refer.1 == row.id {
                    tags_vec.push(next_tag.take().unwrap().0);
                }
            }

            if !tags_done && next_tag.is_none() {
                loop {
                    if let Some(tag) = tags_iter.next() {
                        if tag.1 == row.id {
                            tags_vec.push(tag.0);
                        } else {
                            next_tag = Some(tag);
                            break;
                        }
                    } else {
                        tags_done = true;
                        break;
                    }
                }
            }

            rtn.push(ComposedEntry {
                entry: row,
                custom_field_entries: custom_field_entries_vec.drain().collect(),
                text_entries: text_entries_vec.drain(..).collect(),
                markers: entry_markers_vec.drain(..).collect(),
                tags: tags_vec.drain(..).collect()
            });
        }

        JsonBuilder::new(http::StatusCode::OK)
            .build(Some(rtn))
    }
}

/**
 * POST /entries
 * creates a new entry when given a date for the current user from the session.
 * will also create text and mood entries if given as well
 */
pub async fn handle_post(
    initiator: Initiator,
    db: state::WebDbState,
    posted: web::Json<PostEntryJson>
) -> app_error::Result<impl Responder> {
    let posted = posted.into_inner();
    let conn = &mut *db.get_conn().await?;

    let entry_check = conn.query(
        "select id from entries where day = $1 and owner = $2",
        &[&posted.entry.day, &initiator.user.id]
    ).await?;

    if entry_check.len() != 0 {
        return Err(app_error::ResponseError::EntryExists(
            format!("{}", posted.entry.day)
        ));
    }

    let transaction = conn.transaction().await?;
    let result = transaction.query_one(
        "insert into entries (day, owner) values ($1, $2) returning id, day, owner",
        &[&posted.entry.day, &initiator.user.id]
    ).await?;
    let entry_id: i32 = result.get(0);

    let mut custom_field_entries: HashMap<i32, custom_field_entries::CustomFieldEntry> = HashMap::new();

    if let Some(m) = posted.custom_field_entries {
        for custom_field_entry in m {
            let field = getters::custom_fields::get_via_id(&transaction, custom_field_entry.field, Some(initiator.user.id)).await?;

            db::validation::verifiy_custom_field_entry(&field.config, &custom_field_entry.value)?;

            let value_json = serde_json::to_value(custom_field_entry.value.clone())?;
            let _result = transaction.execute(
                "\
                insert into custom_field_entries (field, value, comment, entry) values \
                ($1, $2, $3, $4)",
                &[&field.id, &value_json, &custom_field_entry.comment, &entry_id]
            ).await?;

            custom_field_entries.insert(field.id, custom_field_entries::CustomFieldEntry {
                field: field.id,
                value: custom_field_entry.value,
                comment: custom_field_entry.comment,
                entry: entry_id
            });
        }
    }

    let mut text_entries: Vec<text_entries::TextEntry> = vec!();

    if let Some(t) = posted.text_entries {
        for text_entry in t {
            let result = transaction.query_one(
                "insert into text_entries (thought, private, entry) values ($1, $2, $3) returning id",
                &[&text_entry.thought, &text_entry.private, &entry_id]
            ).await?;

            text_entries.push(text_entries::TextEntry {
                id: result.get(0),
                thought: text_entry.thought,
                entry: entry_id,
                private: text_entry.private
            });
        }
    }

    let mut entry_tags: Vec<i32> = vec!();

    if let Some(tags) = posted.tags {
        for tag_id in tags {
            let _result = transaction.execute(
                "insert into entries2tags (tag, entry) values ($1, $2)",
                &[&tag_id, &entry_id]
            ).await?;

            entry_tags.push(tag_id);
        }
    }

    let mut entry_markers: Vec<entry_markers::EntryMarker> = vec!();

    if let Some(markers) = posted.markers {
        for marker in markers {
            let result = transaction.query_one(
                "\
                insert into entry_markers (title, comment, entry) values \
                ($1, $2, $3) \
                returning id",
                &[&marker.title, &marker.comment, &entry_id]
            ).await?;

            entry_markers.push(entry_markers::EntryMarker {
                id: result.get(0),
                title: marker.title,
                comment: marker.comment,
                entry: entry_id
            });
        }
    }

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(composed::ComposedEntry {
            entry: entries::Entry {
                id: result.get(0),
                day: result.get(1),
                owner: initiator.user.id,
            },
            tags: entry_tags,
            markers: entry_markers,
            custom_field_entries,
            text_entries
        }))
}