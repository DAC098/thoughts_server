//! handling individual entries based on id

use std::collections::HashMap;

use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;
use chrono::serde::ts_seconds;

pub mod comments;
pub mod audio;

use crate::db::{
    self, 
    tables::{
        permissions,
        entries, 
        custom_field_entries, 
        text_entries,
        entries2tags,
        entry_markers
    }
};
use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::state;
use crate::security::{self, InitiatorLookup, Initiator};
use crate::util;
use crate::components;
use crate::template;
use crate::routing;

#[derive(Deserialize)]
pub struct PutTextEntry {
    id: Option<i32>,
    thought: String,
    private: bool
}

#[derive(Deserialize)]
pub struct PutCustomFieldEntry {
    field: i32,
    value: custom_field_entries::CustomFieldEntryType,
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

/// retrieves a single entry for user when given an id
/// 
/// GET /entries/{id}
/// GET /users/{user_id}/entries/{id}
///
/// returns the requested entry with additional information for the current 
/// user based on the session. auth checks will be performed if reqesting an
/// entry for a nother user
pub async fn handle_get(
    req: HttpRequest,
    security: security::state::WebSecurityState,
    db: state::WebDbState,
    template: template::WebTemplateState<'_>,
    path: web::Path<routing::path::params::EntryPath>
) -> error::Result<impl Responder> {
    let conn = &*db.get_conn().await?;
    let accept_html = response::try_check_if_html_req(&req);
    let lookup = InitiatorLookup::from_request(&security, conn, &req).await?;

    if accept_html {
        return if lookup.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(lookup.unwrap().user))?)
        } else {
            Ok(response::redirect_to_login(&req))
        }
    }
    
    let initiator = lookup.try_into()?;
    let is_private: Option<bool>;
    let owner: i32;

    if let Some(user_id) = path.user_id {
        if !security::permissions::has_permission(
            &*conn, 
            &initiator.user.id, 
            permissions::rolls::USERS_ENTRIES, 
            &[permissions::abilities::READ], 
            Some(&user_id)
        ).await? {
            return Err(error::build::permission_denied(
                "you do not have permission to read this users entries"
            ));
        }

        is_private = Some(false);
        owner = user_id;
    } else {
        if !security::permissions::has_permission(
            &*conn, 
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
            ))
        }

        is_private = None;
        owner = initiator.user.id;
    }

    if let Some(record) = db::composed::ComposedEntry::find_from_entry(conn, &path.entry_id, &is_private).await? {
        if record.entry.owner == owner {
            JsonBuilder::new(http::StatusCode::OK)
                .build(Some(record))
        } else {
            Err(error::build::permission_denied(
                format!("entry owner mis-match. requested entry is not owned by {}", owner)
            ))
        }
    } else {
        Err(error::build::entry_not_found(&path.entry_id))
    }
}

/// updates a given entry with new information
///
/// PUT /entries/{id}
/// 
/// updates the requested entry with new information. it will assume that the
/// new information is the final form and will add/remove/update accordingly
pub async fn handle_put(
    initiator: Initiator,
    db: state::WebDbState,
    path: web::Path<routing::path::params::EntryPath>,
    posted: web::Json<PutComposedEntry>
) -> error::Result<impl Responder> {
    let posted = posted.into_inner();
    let conn = &mut *db.get_conn().await?;
    let created = posted.entry.day.clone();

    if !security::permissions::has_permission(
        &*conn, 
        &initiator.user.id,
        permissions::rolls::ENTRIES,
        &[
            permissions::abilities::READ_WRITE
        ],
        None
    ).await? {
        return Err(error::build::permission_denied(
            "you do not have permission to update entries"
        ));
    }

    security::assert::is_owner_for_entry(conn, &path.entry_id, &initiator.user.id).await?;

    let transaction = conn.transaction().await?;

    let _result = transaction.execute(
        "update entries set day = $1 where id = $2 returning day",
        &[&created, &path.entry_id]
    ).await?;

    let mut rtn = db::composed::ComposedEntry {
        entry: entries::Entry {
            id: path.entry_id,
            day: created,
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
            let field = components::custom_fields::get_via_id(
                &transaction, 
                &custom_field_entry.field, 
                Some(&initiator.user.id)
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
            rtn.custom_field_entries.insert(field.id, custom_field_entries::CustomFieldEntry {
                field: field.id,
                value: custom_field_entry.value,
                comment: util::clone_option(&custom_field_entry.comment),
                entry: path.entry_id
            });
        }

        let _dropped = transaction.query(
            "delete from custom_field_entries where entry = $1 and field <> all($2)",
            &[&path.entry_id, &ids]
        ).await?;
    } else {
        rtn.custom_field_entries = custom_field_entries::find_from_entry_hashmap(&transaction, &path.entry_id).await?;
    }

    if let Some(t) = posted.text_entries {
        let mut ids: Vec<i32> = vec!();

        for text_entry in t {
            if let Some(id) = text_entry.id {
                let result = transaction.execute(
                    "update text_entries set thought = $1, private = $2 where id = $3 returning id",
                    &[&text_entry.thought, &text_entry.private, &id]
                ).await?;

                if result == 0 {
                    return Err(error::build::text_entry_not_found(&id));
                }

                ids.push(id);
                rtn.text_entries.push(text_entries::TextEntry {
                    id,
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
                rtn.text_entries.push(text_entries::TextEntry {
                    id: result.get(0),
                    thought: text_entry.thought,
                    entry: path.entry_id,
                    private: text_entry.private
                })
            }
        }

        let _dropped = transaction.query(
            "delete from text_entries where entry = $1 and id <> all($2)",
            &[&path.entry_id, &ids]
        ).await?;
    } else {
        let is_private = None;
        rtn.text_entries = text_entries::find_from_entry(&transaction, &path.entry_id, &is_private).await?;
    }

    if let Some(tags) = posted.tags {
        for tag_id in &tags {
            let _result = transaction.execute(
                "\
                insert into entries2tags (tag, entry) \
                values ($1, $2) \
                on conflict on constraint unique_entry_tag do update \
                set tag = excluded.tag",
                &[&tag_id, &path.entry_id]
            ).await?;
        }

        let _dropped = transaction.execute(
            "delete from entries2tags where entry = $1 and tag <> all($2)",
            &[&path.entry_id, &tags]
        ).await?;

        rtn.tags = tags;
    } else {
        rtn.tags = entries2tags::find_id_from_entry(&transaction, &path.entry_id).await?;
    }

    if let Some(markers) = posted.markers {
        let mut ids: Vec<i32> = vec!();

        for marker in markers {
            if let Some(id) = marker.id {
                let result = transaction.execute(
                    "update entry_markers set title = $1, comment = $2 where id = $3 and entry = $4",
                    &[&marker.title, &marker.comment, &id, &path.entry_id]
                ).await?;

                if result == 0 {
                    return Err(error::build::entry_marker_not_found(&id));
                }

                ids.push(id);
                rtn.markers.push(entry_markers::EntryMarker {
                    id,
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
                rtn.markers.push(entry_markers::EntryMarker {
                    id: result.get(0),
                    title: marker.title,
                    comment: marker.comment,
                    entry: path.entry_id
                });
            }
        }

        let _dropped = transaction.execute(
            "delete from entry_markers where entry = $1 and id <> all($2)",
            &[&path.entry_id, &ids]
        ).await?;
    } else {
        rtn.markers = entry_markers::find_from_entry(&transaction, &path.entry_id).await?;
    }

    transaction.commit().await?;
    
    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(rtn))
}

/// deletes the given entry id
/// 
/// DELETE /entries/{id}
///
/// checks to make sure that the entry is owned by the current user before
/// deleting. all data associated with the entry will be removed first and
/// then the actual entry will be deleted
pub async fn handle_delete(
    initiator: Initiator,
    db: state::WebDbState,
    path: web::Path<routing::path::params::EntryPath>
) -> error::Result<impl Responder> {
    let conn = &mut *db.get_conn().await?;

    if !security::permissions::has_permission(
        &*conn, 
        &initiator.user.id, 
        permissions::rolls::ENTRIES, 
        &[
            permissions::abilities::READ_WRITE
        ], 
        None
    ).await? {
        return Err(error::build::permission_denied(
            "you do not have permission to delete entries"
        ));
    }

    let transaction = conn.transaction().await?;

    let Some(_record) = transaction.query_opt(
        "select id, owner from entries where id = $1 and owner = $2",
        &[&path.entry_id, &initiator.user.id]
    ).await? else {
        return Err(error::build::entry_not_found(&path.entry_id));
    };

    let _text_result = transaction.execute(
        "delete from text_entries where entry = $1",
        &[&path.entry_id]
    ).await?;

    let _custom_field_entries_result = transaction.execute(
        "delete from custom_field_entries where entry = $1",
        &[&path.entry_id]
    ).await?;

    let _tag_result = transaction.execute(
        "delete from entries2tags where entry = $1",
        &[&path.entry_id]
    ).await?;

    let _marker_result = transaction.execute(
        "delete from entry_markers where entry = $1",
        &[&path.entry_id]
    ).await?;

    let _entry_result = transaction.execute(
        "delete from entries where id = $1",
        &[&path.entry_id]
    ).await?;

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(None::<()>)
}
