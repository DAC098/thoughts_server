use std::collections::{HashMap};

use actix_web::{web, http, HttpRequest, Responder};
use actix_session::{Session};
use serde::{Deserialize};

use crate::db;
use crate::response;
use crate::state;
use crate::request::from;
use crate::json;
use crate::security;
use crate::util;
use crate::getters;

use response::error as app_error;

#[derive(Deserialize)]
pub struct PutTextEntryJson {
    id: Option<i32>,
    thought: String,
    private: bool
}

#[derive(Deserialize)]
pub struct PutCustomFieldEntryJson {
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
pub struct PutEntryJson {
    created: chrono::DateTime<chrono::Utc>,
    tags: Option<Vec<i32>>,
    markers: Option<Vec<PutEntryMarker>>,
    custom_field_entries: Option<Vec<PutCustomFieldEntryJson>>,
    text_entries: Option<Vec<PutTextEntryJson>>
}

#[derive(Deserialize)]
pub struct EntryPath {
    entry_id: i32
}

/**
 * GET /entries/{id}
 * returns the requested entry with additional information for the current user
 * given the session
 */
pub async fn handle_get(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>,
    path: web::Path<EntryPath>
) -> app_error::Result<impl Responder> {
    let conn = &*app.get_conn().await?;
    let accept_html = response::check_if_html_req(&req, true).unwrap();
    let initiator_opt = from::get_initiator(conn, &session).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(Some(initiator_opt.unwrap().user)))
        } else {
            let redirect = format!("/auth/login?jump_to=/entries/{}", path.entry_id);
            Ok(response::redirect_to_path(redirect.as_str()))
        }
    } else if initiator_opt.is_none() {
        Err(app_error::ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();

        if let Some(entry) = json::search_entry(conn, path.entry_id, None).await? {
            if entry.owner == initiator.user.get_id() {
                Ok(response::json::respond_json(
                    http::StatusCode::OK, 
                    response::json::MessageDataJSON::build(
                        "successful",
                        entry
                    )
                ))
            } else {
                Err(app_error::ResponseError::PermissionDenied(
                    format!("you do not have permission to view this users entry as you are not the owner")
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
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    path: web::Path<EntryPath>,
    posted_cntr: web::Json<PutEntryJson>
) -> app_error::Result<impl Responder> {
    let posted = posted_cntr.into_inner();
    let conn = &mut *app.get_conn().await?;
    let created = posted.created.clone();
    security::assert::is_owner_for_entry(conn, path.entry_id, initiator.user.get_id()).await?;

    let transaction = conn.transaction().await?;
    let result = transaction.query_one(
        "update entries set day = $1 where id = $2 returning day",
        &[&created, &path.entry_id]
    ).await?;
    let mut rtn = json::EntryJson {
        id: path.entry_id,
        created: result.get(0),
        tags: vec!(),
        markers: vec!(),
        custom_field_entries: HashMap::new(),
        text_entries: vec!(),
        owner: initiator.user.get_id()
    };

    if let Some(m) = posted.custom_field_entries {
        let mut ids: Vec<i32> = vec!();

        for custom_field_entry in m {
            let field = getters::custom_fields::get_via_id(
                &transaction, 
                custom_field_entry.field, 
                Some(initiator.user.id)
            ).await?;

            db::custom_fields::verifiy(&field.config, &custom_field_entry.value)?;

            let value_json = serde_json::to_value(custom_field_entry.value.clone())?;
            let _result = transaction.execute(
                r#"
                insert into custom_field_entries (field, value, comment, entry) values
                ($1, $2, $3, $4)
                on conflict on constraint entry_field_key do update
                set value = excluded.value,
                    comment = excluded.comment
                "#,
                &[&field.id, &value_json, &custom_field_entry.comment, &path.entry_id]
            ).await?;

            ids.push(field.id);
            rtn.custom_field_entries.insert(field.id, json::CustomFieldEntryJson {
                field: field.id,
                name: field.name,
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
        rtn.custom_field_entries = json::search_custom_field_entries(&transaction, &path.entry_id).await?;
    }

    if let Some(t) = posted.text_entries {
        let mut ids: Vec<i32> = vec!();

        for text_entry in t {
            if let Some(id) = text_entry.id {
                let check = transaction.query(
                    r#"
                    select entries.owner
                    from text_entries
                    join entries on text_entries.entry = entries.id
                    where text_entries.id = $1
                    "#,
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
                rtn.text_entries.push(json::TextEntryJson {
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
                rtn.text_entries.push(json::TextEntryJson {
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
        rtn.text_entries = json::search_text_entries(&transaction, &path.entry_id, None).await?;
    }

    if let Some(tags) = posted.tags {
        let mut ids: Vec<i32> = vec!();

        for tag_id in tags {
            let result = transaction.query_one(
                r#"
                insert into entries2tags (tag, entry) 
                values ($1, $2)
                on conflict on constraint unique_entry_tag do update
                set tag = excluded.tag
                returning id
                "#,
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
        rtn.tags = json::search_tag_entries(&transaction, &path.entry_id).await?;
    }

    if let Some(markers) = posted.markers {
        let mut ids: Vec<i32> = vec!();

        for marker in markers {
            if let Some(id) = marker.id {
                let check = transaction.query(
                    r#"
                    select entries.owner
                    from entry_markers
                    join entries on entry_markers.entry = entries.id
                    where entry_markers.entry = $1
                    "#,
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
                    r#"
                    update entry_markers set title = $1, comment = $2 where id = $3
                    "#,
                    &[&marker.title, &marker.comment, &id]
                ).await?;

                ids.push(id);
                rtn.markers.push(json::EntryMarker {
                    id: id,
                    title: marker.title,
                    comment: marker.comment
                });
            } else {
                let result = transaction.query_one(
                    r#"
                    insert into entry_markers (title, comment, entry) values
                    ($1, $2, $3)
                    returning id
                    "#,
                    &[&marker.title, &marker.comment, &path.entry_id]
                ).await?;

                ids.push(result.get(0));
                rtn.markers.push(json::EntryMarker {
                    id: result.get(0),
                    title: marker.title,
                    comment: marker.comment
                });
            }
        }
    } else {
        rtn.markers = json::search_entry_markers(&transaction, &path.entry_id).await?;
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
    initiator: from::Initiator,
    app_data: web::Data<state::AppState>,
    path: web::Path<EntryPath>
) -> app_error::Result<impl Responder> {
    let app = app_data.into_inner();
    let mut conn = app.get_conn().await?;
    let transaction = conn.transaction().await?;

    let check = transaction.query(
        "select id, owner from entries where id = $1",
        &[&path.entry_id]
    ).await?;
    let mut invalid_entries: Vec<i32> = vec!();

    for row in check {
        if row.get::<usize, i32>(1) != initiator.user.get_id() {
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