//! handling listing and creating entries

use std::fmt::Write;
//use std::pin::{Pin};
//use std::task::{Context, Poll};

use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;
use chrono::{DateTime, Utc};
use chrono::serde::ts_seconds;
use tokio_postgres::GenericClient;
//use futures::{pin_mut, Stream, TryStreamExt, future};
use futures::future;

pub mod entry_id;

use crate::db::{
    self,
    tables::{
        permissions,
        custom_field_entries,
    },
};
use crate::net::http::{error, response::{self, json::JsonBuilder}};
use crate::routing;
use crate::state;
use crate::security::{self, InitiatorLookup, Initiator};
use crate::components::{self, entries::schema};
use crate::template;

#[derive(Deserialize)]
pub struct EntriesQuery {
    from: Option<String>,
    to: Option<String>,
    tags: Option<String>,
    from_marker: Option<i32>,
    to_marker: Option<i32>,
}

/// retrieves entry date from marker id
pub async fn get_marker_date(
    conn: &impl GenericClient,
    owner: &i32,
    marker: &i32,
) -> error::Result<Option<DateTime<Utc>>> {
    let marker_check = conn.query(
        "\
        select entries.day \
        from entries \
        join entry_markers on entries.id = entry_markers.entry \
        where entry_markers.id = $1 and entries.owner = $2",
        &[marker, owner]
    ).await?;

    if marker_check.is_empty() {
        Ok(None)
    } else {
        Ok(Some(marker_check[0].get::<usize, DateTime<Utc>>(0)))
    }
}

/// searches entries for a user
///
/// GET /entries
/// GET /users/{user_id}/entries
///
/// returns the root html if requesting html. otherwise will send back a list of
/// available and allowed entries for the current user from the session. if
/// attempting to access another users entries auth checks will be performed
/// to see if they are allowed to view this information.
pub async fn handle_get(
    req: HttpRequest,
    security: security::state::WebSecurityState,
    db: state::WebDbState,
    template: template::WebTemplateState<'_>,
    info: web::Query<EntriesQuery>,
    path: web::Path<routing::path::params::OptUserPath>,
) -> error::Result<impl Responder> {
    let info = info.into_inner();
    let pool_conn = db.pool.get().await?;
    let accept_html = response::try_check_if_html_req(&req);
    let lookup = InitiatorLookup::from_request(&security, &*pool_conn, &req).await?;

    if accept_html {
        return if lookup.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(lookup.unwrap().user))?)
        } else {
            Ok(response::redirect_to_login(&req))
        }
    }

    let owner: i32;
    let is_private: Option<bool>;
    let initiator = lookup.try_into()?;

    if let Some(user_id) = path.user_id {
        if !security::permissions::has_permission(
            &*pool_conn, 
            &initiator.user.id, 
            permissions::rolls::USERS_ENTRIES, 
            &[permissions::abilities::READ], 
            Some(&user_id)
        ).await? {
            return Err(error::build::permission_denied(
                "you do not have permission to view this users entries"
            ));
        }

        is_private = Some(false);
        owner = user_id;
    } else {
        if !security::permissions::has_permission(
            &*pool_conn, 
            &initiator.user.id, 
            permissions::rolls::ENTRIES, 
            &[
                permissions::abilities::READ,
                permissions::abilities::READ_WRITE
            ],
            None
        ).await? {
            return Err(error::build::permission_denied(
                "you do not have permission to read entries"
            ));
        }

        is_private = None;
        owner = initiator.user.id;
    }

    let mut results = {
        let mut entries_where: String = String::new();

        write!(&mut entries_where, "entries.owner = {}", owner)?;

        if let Some(from_marker) = info.from_marker {
            let Some(day) = get_marker_date(&*pool_conn, &owner, &from_marker).await? else {
                return Err(error::build::bad_request(
                    format!("from makrer id given does not exist: {}", from_marker)
                ));
            };

            write!(&mut entries_where, " and entries.day >= '{}'", day.to_rfc3339())?;
        } else if let Some(from) = routing::query::get_date(&info.from)? {
            write!(&mut entries_where, " and entries.day >= '{}'", from.to_rfc3339())?;
        }

        if let Some(to_marker) = info.to_marker {
            let Some(day) = get_marker_date(&*pool_conn, &owner, &to_marker).await? else {
                return Err(error::build::bad_request(
                    format!("to marker id given does not exist: {}", to_marker)
                ));
            };

            write!(&mut entries_where, "and entries.day <= '{}'", day.to_rfc3339())?;
        } else if let Some(to) = routing::query::get_date(&info.to)? {
            write!(&mut entries_where, " and entries.day <= '{}'", to.to_rfc3339())?;
        }

        if let Some(tags) = routing::query::get_tags(&info.tags) {
            write!(
                &mut entries_where,
                " and entries.id in (select entry from entries2tags where tag in ({}))", 
                tags.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(",")
            )?;
        }

        let rows_statement = format!(
            "\
            select id, \
                   day, \
                   created, \
                   updated, \
                   deleted, \
                   owner \
            from entries \
            where {} \
            order by day desc",
            entries_where
        );

        let custom_field_entries_statement = format!(
            "\
            select custom_field_entries.field, \
                   custom_field_entries.value, \
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
                   entry_markers.entry \
            from entry_markers \
            join entries on entry_markers.entry = entries.id \
            where {} \
            order by entries.day desc, entry_markers.id",
            entries_where
        );

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


        let text_entries_statement = {
            let mut query_str = format!(
                "\
                select text_entries.entry, \
                       count(text_entries.id) \
                from text_entries \
                join entries on text_entries.entry = entries.id \
                where {}",
                entries_where
            );

            if let Some(is_private) = is_private {
                write!(&mut query_str, " and text_entries.private = {}", if is_private { "true" } else { "false" })?;
            }

            write!(
                &mut query_str,
                " \
                group by text_entries.entry \
                order by entries.day desc, text_entries.id"
            )?;

            query_str
        };

        let audio_entries_statement = {
            let mut query_str = format!(
                "\
                select audio_entries.entry, \
                       count(audio_entries.id) \
                from audio_entries \
                join entries on audio_entries.entry = entries.id \
                where {}",
                entries_where
            );

            if let Some(is_private) = is_private {
                write!(&mut query_str, " and audio_entries.private = {}", if is_private { "true" } else { "false" })?;
            }

            write!(
                &mut query_str,
                " \
                group by audio_entries.entry \
                order by entries.day desc"
            )?;

            query_str
        };

        // video entries

        // files

        future::try_join_all(vec![
            (*pool_conn).query(rows_statement.as_str(), &[]),
            (*pool_conn).query(custom_field_entries_statement.as_str(), &[]),
            (*pool_conn).query(entry_markers_statement.as_str(), &[]),
            (*pool_conn).query(tags_statement.as_str(), &[]),
            (*pool_conn).query(text_entries_statement.as_str(), &[]),
            (*pool_conn).query(audio_entries_statement.as_str(), &[]),
        ]).await?
    };

    let audio = results.pop()
        .unwrap();
    let mut audio_iter = audio.iter()
        .map(|row| (
            row.get::<usize, i32>(0), 
            row.get::<usize, i64>(1)
        ));
    let text = results.pop()
        .unwrap();
    let mut text_iter = text.iter()
        .map(|row| (
            row.get::<usize, i32>(0),
            row.get::<usize, i64>(1)
        ));

    let tags = results.pop()
        .unwrap();
    let mut tags_iter = tags.iter()
        .map(|row| (
            row.get::<usize, i32>(0),
            row.get::<usize, i32>(1)
        ));
    let entry_markers = results.pop()
        .unwrap();
    let mut entry_markers_iter = entry_markers.iter()
        .map(|row| (
            row.get::<usize, i32>(2),
            schema::ListMarker {
                id: row.get(0),
                title: row.get(1),
            }
        ));
    let custom_field_entries = results.pop()
        .unwrap();
    let mut custom_field_entries_iter = custom_field_entries.iter()
        .map(|row| (
            row.get::<usize, i32>(2),
            schema::ListCustomField {
                field: row.get(0),
                value: serde_json::from_value(row.get(1)).unwrap(),
            }
        ));
    let rows = results.pop()
        .unwrap();
    let rows_iter = rows.iter()
        .map(|row| schema::ListEntry {
            id: row.get(0),
            day: row.get(1),
            created: row.get(2),
            updated: row.get(3),
            deleted: row.get(4),
            owner: row.get(5),
            tags: Vec::new(),
            markers: Vec::new(),
            fields: Vec::new(),
            text: 0,
            audio: 0,
            video: 0,
            files: 0,
        });

    let mut rtn = Vec::with_capacity(rows.len());

    let mut audio_done = false;
    let mut fields_done = false;
    let mut markers_done = false;
    let mut text_done = false;
    let mut tags_done = false;
    let mut next_audio_count: Option<(i32, i64)> = None;
    let mut next_text_count: Option<(i32, i64)> = None;
    let mut next_custom_field_entry: Option<(i32, schema::ListCustomField)> = None;
    let mut next_entry_marker: Option<(i32, schema::ListMarker)> = None;
    let mut next_tag: Option<(i32, i32)> = None;

    for mut row in rows_iter {
        if let Some(refer) = next_audio_count.as_ref() {
            if refer.0 == row.id {
                let taken = next_audio_count.take().unwrap();
                row.audio = taken.1;
            }
        }

        if !audio_done && next_audio_count.is_none() {
            if let Some(count) = audio_iter.next() {
                if count.0 == row.id {
                    row.audio = count.1;
                } else {
                    next_audio_count = Some(count);
                }
            } else {
                audio_done = true;
            }
        }

        if let Some(refer) = next_text_count.as_ref() {
            if refer.0 == row.id {
                let taken = next_text_count.take().unwrap();
                row.text = taken.1;
            }
        }

        if !text_done && next_text_count.is_none() {
            if let Some(count) = text_iter.next() {
                if count.0 == row.id {
                    row.text = count.1;
                } else {
                    next_text_count = Some(count);
                }
            } else {
                text_done = true;
            }
        }

        if let Some(refer) = next_custom_field_entry.as_ref() {
            if refer.0 == row.id {
                let taken = next_custom_field_entry.take().unwrap();
                row.fields.push(taken.1);
            }
        }

        if !fields_done && next_custom_field_entry.is_none() {
            loop {
                if let Some(tup) = custom_field_entries_iter.next() {
                    if tup.0 == row.id {
                        row.fields.push(tup.1);
                    } else {
                        next_custom_field_entry = Some(tup);
                        break;
                    }
                } else {
                    fields_done = true;
                    break;
                }
            }
        }

        if let Some(refer) = next_entry_marker.as_ref() {
            if refer.0 == row.id {
                let taken = next_entry_marker.take().unwrap();
                row.markers.push(taken.1);
            }
        }

        if !markers_done && next_entry_marker.is_none() {
            loop {
                if let Some(tup) = entry_markers_iter.next() {
                    if tup.0 == row.id {
                        row.markers.push(tup.1);
                    } else {
                        next_entry_marker = Some(tup);
                        break;
                    }
                } else {
                    markers_done = true;
                    break;
                }
            }
        }

        if let Some(refer) = next_tag.as_ref() {
            if refer.1 == row.id {
                row.tags.push(next_tag.take().unwrap().0);
            }
        }

        if !tags_done && next_tag.is_none() {
            loop {
                if let Some(tag) = tags_iter.next() {
                    if tag.1 == row.id {
                        row.tags.push(tag.0);
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

        rtn.push(row);
    }

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(rtn))
}

#[derive(Deserialize)]
pub struct PostTextEntryJson {
    thought: String,
    private: bool
}

#[derive(Deserialize)]
pub struct PostCustomFieldEntryJson {
    field: i32,
    value: custom_field_entries::CustomFieldEntryType,
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

/// posts entries for a given user
///
/// POST /entries
///
/// creates a new entry when given a date for the current user from the 
/// session. 
pub async fn handle_post(
    initiator: Initiator,
    db: state::WebDbState,
    posted: web::Json<PostEntryJson>
) -> error::Result<impl Responder> {
    let posted = posted.into_inner();
    let conn = &mut *db.get_conn().await?;

    if !security::permissions::has_permission(
        &*conn, 
        &initiator.user.id, 
        permissions::rolls::ENTRIES, 
        &[permissions::abilities::READ_WRITE], 
        None
    ).await? {
        return Err(error::build::permission_denied(
            "you do not have permission to create entries"
        ));
    }

    let entry_check = conn.query(
        "select id from entries where day = $1 and owner = $2",
        &[&posted.entry.day, &initiator.user.id]
    ).await?;

    if entry_check.len() != 0 {
        return Err(error::build::entry_exists(
            posted.entry.day.to_string()
        ));
    }

    let transaction = conn.transaction().await?;
    let created = Utc::now();

    let result = transaction.query_one(
        "insert into entries (day, owner, created) values ($1, $2) returning id",
        &[&posted.entry.day, &initiator.user.id, &created]
    ).await?;

    let entry_id: i32 = result.get(0);

    let mut custom_field_entries = Vec::new();

    if let Some(m) = posted.custom_field_entries {
        custom_field_entries.reserve(m.len());

        for custom_field_entry in m {
            let field = components::custom_fields::get_via_id(
                &transaction, 
                &custom_field_entry.field, 
                Some(&initiator.user.id)
            ).await?;

            db::validation::verifiy_custom_field_entry(&field.config, &custom_field_entry.value)?;

            let value_json = serde_json::to_value(custom_field_entry.value.clone())?;
            let _result = transaction.execute(
                "\
                insert into custom_field_entries (field, value, comment, entry) values \
                ($1, $2, $3, $4)",
                &[&field.id, &value_json, &custom_field_entry.comment, &entry_id]
            ).await?;

            custom_field_entries.push(schema::CustomField {
                field: field.id,
                value: custom_field_entry.value,
                comment: custom_field_entry.comment,
            });
        }
    }

    let mut text_entries = Vec::new();

    if let Some(t) = posted.text_entries {
        text_entries.reserve(t.len());

        for text_entry in t {
            let result = transaction.query_one(
                "insert into text_entries (thought, private, entry) values ($1, $2, $3) returning id",
                &[&text_entry.thought, &text_entry.private, &entry_id]
            ).await?;

            text_entries.push(schema::Text {
                id: result.get(0),
                thought: text_entry.thought,
                private: text_entry.private
            });
        }
    }

    let mut entry_tags: Vec<i32> = Vec::new();

    if let Some(tags) = posted.tags {
        entry_tags.reserve(tags.len());

        for tag_id in tags {
            let _result = transaction.execute(
                "insert into entries2tags (tag, entry) values ($1, $2)",
                &[&tag_id, &entry_id]
            ).await?;

            entry_tags.push(tag_id);
        }
    }

    let mut entry_markers = Vec::new();

    if let Some(markers) = posted.markers {
        entry_markers.reserve(markers.len());

        for marker in markers {
            let result = transaction.query_one(
                "\
                insert into entry_markers (title, comment, entry) values \
                ($1, $2, $3) \
                returning id",
                &[&marker.title, &marker.comment, &entry_id]
            ).await?;

            entry_markers.push(schema::Marker {
                id: result.get(0),
                title: marker.title,
                comment: marker.comment,
            });
        }
    }

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(schema::Entry {
            id: result.get(0),
            day: posted.entry.day,
            created,
            updated: None,
            deleted: None,
            owner: initiator.user.id.clone(),
            tags: entry_tags,
            markers: entry_markers,
            fields: custom_field_entries,
            text: text_entries,
            audio: Vec::new(),
        }))
}
