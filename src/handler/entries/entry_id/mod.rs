use std::collections::HashMap;

use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;
use chrono::serde::ts_seconds;

use tlib::db;

pub mod comments;
pub mod audio;

use crate::response;
use crate::state;
use crate::request::{initiator_from_request, Initiator};
use crate::security;
use crate::util;
use crate::getters;

use response::error as app_error;

#[derive(Deserialize)]
pub struct PutTextEntry {
    id: Option<i32>,
    thought: String,
    private: bool
}

#[derive(Deserialize)]
pub struct PutCustomFieldEntry {
    field: i32,
    value: db::custom_field_entries::CustomFieldEntryType,
    comment: Option<String>
}

#[derive(Deserialize)]
pub struct PutEntryMarker {
    id: Option<i32>,
    title: String,
    comment: Option<String>
}

#[derive(Deserialize)]
pub struct PutEntry {
    #[serde(with = "ts_seconds")]
    day: chrono::DateTime<chrono::Utc>
}

#[derive(Deserialize)]
pub struct PutComposedEntry {
    entry: PutEntry,
    tags: Option<Vec<i32>>,
    markers: Option<Vec<PutEntryMarker>>,
    custom_field_entries: Option<Vec<PutCustomFieldEntry>>,
    text_entries: Option<Vec<PutTextEntry>>
}

#[derive(Deserialize)]
pub struct EntryPath {
    user_id: Option<i32>,
    entry_id: i32
}

/**
 * GET /entries/{id}
 * returns the requested entry with additional information for the current user
 * given the session
 */
pub async fn handle_get(
    req: HttpRequest,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
    path: web::Path<EntryPath>
) -> app_error::Result<impl Responder> {
    let conn = &*db.get_conn().await?;
    let accept_html = response::try_check_if_html_req(&req);
    let initiator_opt = initiator_from_request(conn, &req).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(initiator_opt.unwrap().user))?)
        } else {
            Ok(response::redirect_to_login(&req))
        }
    } else if initiator_opt.is_none() {
        Err(app_error::ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();
        let is_private: Option<bool>;
        let owner: i32;

        if let Some(user_id) = path.user_id {
            security::assert::permission_to_read(conn, &initiator.user.id, &user_id).await?;
            is_private = Some(false);
            owner = user_id;
        } else {
            is_private = None;
            owner = initiator.user.id;
        }

        if let Some(record) = db::composed::ComposedEntry::find_from_entry(conn, &path.entry_id, &is_private).await? {
            if record.entry.owner == owner {
                Ok(response::json::respond_json(
                    http::StatusCode::OK, 
                    response::json::MessageDataJSON::build(
                        "successful",
                        record
                    )
                ))
            } else {
                Err(app_error::ResponseError::PermissionDenied(
                    format!("entry owner mis-match. requested entry is not owned by {}", owner)
                ))
            }
        } else {
            Err(app_error::ResponseError::EntryNotFound(path.entry_id))
        }
    }
}

/**
 * PUT /entries/{id}
 * updates the requested entry with mood or text entries for the current
 * user
 */
pub async fn handle_put(
    initiator: Initiator,
    db: state::WebDbState,
    path: web::Path<EntryPath>,
    posted: web::Json<PutComposedEntry>
) -> app_error::Result<impl Responder> {
    let posted = posted.into_inner();
    let conn = &mut *db.get_conn().await?;
    let created = posted.entry.day.clone();
    security::assert::is_owner_for_entry(conn, &path.entry_id, &initiator.user.id).await?;

    let transaction = conn.transaction().await?;
    let result = transaction.query_one(
        "update entries set day = $1 where id = $2 returning day",
        &[&created, &path.entry_id]
    ).await?;
    let mut rtn = db::composed::ComposedEntry {
        entry: db::entries::Entry {
            id: path.entry_id,
            day: result.get(0),
            owner: initiator.user.id
        },
        tags: vec!(),
        markers: vec!(),
        custom_field_entries: HashMap::new(),
        text_entries: vec!(),
    };

    if let Some(m) = posted.custom_field_entries {
        let mut ids: Vec<i32> = vec!();

        for custom_field_entry in m {
            let field = getters::custom_fields::get_via_id(
                &transaction, 
                custom_field_entry.field, 
                Some(initiator.user.id)
            ).await?;

            db::validation::verifiy_custom_field_entry(&field.config, &custom_field_entry.value)?;

            let value_json = serde_json::to_value(custom_field_entry.value.clone())?;
            let _result = transaction.execute(
                "\
                insert into custom_field_entries (field, value, comment, entry) \
                values ($1, $2, $3, $4) \
                on conflict on constraint entry_field_key do update \
                set value = excluded.value, \
                    comment = excluded.comment",
                &[&field.id, &value_json, &custom_field_entry.comment, &path.entry_id]
            ).await?;

            ids.push(field.id);
            rtn.custom_field_entries.insert(field.id, db::custom_field_entries::CustomFieldEntry {
                field: field.id,
                value: custom_field_entry.value,
                comment: util::clone_option(&custom_field_entry.comment),
                entry: path.entry_id
            });
        }

        let left_over = transaction.query(
            "select field from custom_field_entries where entry = $1 and not (field = any($2))",
            &[&path.entry_id, &ids]
        ).await?;

        if left_over.len() > 0 {
            let mut to_delete = Vec::<i32>::with_capacity(left_over.len());

            for row in left_over {
                to_delete.push(row.get(0));
            }

            let _result = transaction.execute(
                "delete from custom_field_entries where field = any($1) and entry = $2",
                &[&to_delete, &path.entry_id]
            ).await?;
        }
    } else {
        rtn.custom_field_entries = db::custom_field_entries::find_from_entry_hashmap(&transaction, &path.entry_id).await?;
    }

    if let Some(t) = posted.text_entries {
        let mut ids: Vec<i32> = vec!();

        for text_entry in t {
            if let Some(id) = text_entry.id {
                let check = transaction.query(
                    "\
                    select entries.owner \
                    from text_entries \
                    join entries on text_entries.entry = entries.id \
                    where text_entries.id = $1",
                    &[&id]
                ).await?;

                if check.len() == 0 {
                    return Err(app_error::ResponseError::TextEntryNotFound(id));
                }

                if check[0].get::<usize, i32>(0) != initiator.user.id {
                    return Err(app_error::ResponseError::PermissionDenied(
                        format!("you do not have permission to modify another users text entry. text entry: {}", id)
                    ));
                }

                let result = transaction.query_one(
                    "update text_entries set thought = $1, private = $2 where id = $3 returning id",
                    &[&text_entry.thought, &text_entry.private, &id]
                ).await?;

                ids.push(id);
                rtn.text_entries.push(db::text_entries::TextEntry {
                    id: result.get(0),
                    thought: text_entry.thought,
                    entry: path.entry_id,
                    private: text_entry.private
                });
            } else {
                let result = transaction.query_one(
                    "insert into text_entries (thought, private, entry) values ($1, $2, $3) returning id",
                    &[&text_entry.thought, &text_entry.private, &path.entry_id]
                ).await?;

                ids.push(result.get(0));
                rtn.text_entries.push(db::text_entries::TextEntry {
                    id: result.get(0),
                    thought: text_entry.thought,
                    entry: path.entry_id,
                    private: text_entry.private
                })
            }
        }

        let left_over = transaction.query(
            "select id from text_entries where entry = $1 and not (id = any($2))",
            &[&path.entry_id, &ids]
        ).await?;

        if left_over.len() > 0 {
            let mut to_delete = Vec::<i32>::with_capacity(left_over.len());

            for row in left_over {
                to_delete.push(row.get(0));
            }

            let _result = transaction.execute(
                "delete from text_entries where id = any($1)",
                &[&to_delete]
            ).await?;
        }
    } else {
        let is_private = None::<bool>;
        rtn.text_entries = db::text_entries::find_from_entry(&transaction, &path.entry_id, &is_private).await?;
    }

    if let Some(tags) = posted.tags {
        let mut ids: Vec<i32> = vec!();

        for tag_id in tags {
            let result = transaction.query_one(
                "\
                insert into entries2tags (tag, entry) \
                values ($1, $2) \
                on conflict on constraint unique_entry_tag do update \
                set tag = excluded.tag \
                returning id",
                &[&tag_id, &path.entry_id]
            ).await?;

            ids.push(result.get(0));
            rtn.tags.push(tag_id);
        }

        let left_over = transaction.query(
            "select id from entries2tags where entry = $1 and not (id = any ($2))",
            &[&path.entry_id, &ids]
        ).await?;

        if left_over.len() > 0 {
            let mut to_delete: Vec<i32> = Vec::with_capacity(left_over.len());

            for row in left_over {
                to_delete.push(row.get(0));
            }

            let _result = transaction.execute(
                "delete from entries2tags where id = any($1)",
                &[&to_delete]
            ).await?;
        }
    } else {
        rtn.tags = db::entries2tags::find_id_from_entry(&transaction, &path.entry_id).await?;
    }

    if let Some(markers) = posted.markers {
        let mut ids: Vec<i32> = vec!();

        for marker in markers {
            if let Some(id) = marker.id {
                let check = transaction.query(
                    "\
                    select entries.owner \
                    from entry_markers \
                    join entries on entry_markers.entry = entries.id \
                    where entry_markers.entry = $1",
                    &[&path.entry_id]
                ).await?;

                if check.is_empty() {
                    return Err(app_error::ResponseError::EntryMarkerNotFound(id));
                }

                if check[0].get::<usize, i32>(0) != initiator.user.id {
                    return Err(app_error::ResponseError::PermissionDenied(
                        format!("you do not have permission to modify another users entry marker. text entry: {}", id)
                    ));
                }

                transaction.execute(
                    "update entry_markers set title = $1, comment = $2 where id = $3",
                    &[&marker.title, &marker.comment, &id]
                ).await?;

                ids.push(id);
                rtn.markers.push(db::entry_markers::EntryMarker {
                    id: id,
                    title: marker.title,
                    comment: marker.comment,
                    entry: path.entry_id
                });
            } else {
                let result = transaction.query_one(
                    "\
                    insert into entry_markers (title, comment, entry) \
                    values ($1, $2, $3) \
                    returning id",
                    &[&marker.title, &marker.comment, &path.entry_id]
                ).await?;

                ids.push(result.get(0));
                rtn.markers.push(db::entry_markers::EntryMarker {
                    id: result.get(0),
                    title: marker.title,
                    comment: marker.comment,
                    entry: path.entry_id
                });
            }
        }
    } else {
        rtn.markers = db::entry_markers::find_from_entry(&transaction, &path.entry_id).await?;
    }

    transaction.commit().await?;
    
    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful",
            rtn
        )
    ))
}

/**
 * DELETE /entries/{id}
 */
pub async fn handle_delete(
    initiator: Initiator,
    db: state::WebDbState,
    path: web::Path<EntryPath>
) -> app_error::Result<impl Responder> {
    let conn = &mut *db.get_conn().await?;
    let transaction = conn.transaction().await?;

    let check = transaction.query(
        "select id, owner from entries where id = $1",
        &[&path.entry_id]
    ).await?;
    let mut invalid_entries: Vec<i32> = vec!();

    for row in check {
        if row.get::<usize, i32>(1) != initiator.user.id {
            invalid_entries.push(row.get(0));
        }

        if invalid_entries.len() > 0 {
            return Err(app_error::ResponseError::PermissionDenied(
                format!("you are not allowed to delete entries owned by another user. entries ({:?})", invalid_entries)
            ));
        }
    }
    
    let _text_result = transaction.execute(
        "delete from text_entries where entry = $1",
        &[&path.entry_id]
    ).await?;

    let _custom_field_entries_result = transaction.execute(
        "delete from custom_field_entries where entry = $1",
        &[&path.entry_id]
    ).await?;

    let _entry_result = transaction.execute(
        "delete from entries where id = $1",
        &[&path.entry_id]
    ).await?;

    transaction.commit().await?;

    Ok(response::json::respond_okay())
}