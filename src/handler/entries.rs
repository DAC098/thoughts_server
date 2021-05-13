use actix_web::{web, http, HttpRequest, Responder};
use actix_session::{Session};
use tokio_postgres::{GenericClient};
use serde::{Deserialize};

use crate::db;
use crate::response;
use crate::{error as app_error};
use crate::state;
use crate::request::from;
use crate::json;
use crate::security;

#[derive(Deserialize)]
pub struct PostTextEntryJson {
    thought: String,
    private: bool
}

#[derive(Deserialize)]
pub struct PostMoodEntryJson {
    field_id: i32,
    value: db::mood_entries::MoodEntryType,
    comment: Option<String>
}

#[derive(Deserialize)]
pub struct PostEntryJson {
    created: Option<chrono::DateTime<chrono::Utc>>,
    mood_entries: Option<Vec<PostMoodEntryJson>>,
    text_entries: Option<Vec<PostTextEntryJson>>
}

#[derive(Deserialize)]
pub struct PutTextEntryJson {
    id: Option<i32>,
    thought: String,
    private: bool
}

#[derive(Deserialize)]
pub struct PutMoodEntryJson {
    id: Option<i32>,
    field_id: Option<i32>,
    value: db::mood_entries::MoodEntryType,
    comment: Option<String>
}

#[derive(Deserialize)]
pub struct PutEntryJson {
    created: chrono::DateTime<chrono::Utc>,
    mood_entries: Option<Vec<PutMoodEntryJson>>,
    text_entries: Option<Vec<PutTextEntryJson>>
}

fn clone_string_option(string_opt: &Option<String>) -> Option<String> {
    match string_opt {
        Some(string) => Some(string.clone()),
        None => None
    }
}

async fn get_mood_field_via_id(
    conn: &impl GenericClient,
    initiator: i32,
    field_id: i32,
) -> app_error::Result<db::mood_fields::MoodField> {
    let field = match db::mood_fields::find_id(conn, field_id).await? {
        Some(field) => field,
        None => Err(app_error::ResponseError::MoodFieldNotFound(field_id))?
    };

    if field.get_owner() != initiator {
        return Err(app_error::ResponseError::PermissionDenied(
            format!("you do not haver permission to create a mood entry using this field id: {}", field.get_owner())
        ));
    }

    Ok(field)
}

async fn get_mood_field_via_mood_entry(
    conn: &impl GenericClient,
    initiator: i32,
    mood_id: i32,
) -> app_error::Result<db::mood_fields::MoodField> {
    let result = conn.query(
        r#"
        select mood_entries.field,
               entries.owner 
        from mood_entries 
        join entries on mood_entries.entry = entries.id
        where mood_entries.id = $1
        "#,
        &[&mood_id]
    ).await?;

    if result.len() == 0 {
        return Err(app_error::ResponseError::MoodEntryNotFound(mood_id));
    }

    if result[0].get::<usize,i32>(1) != initiator {
        return Err(app_error::ResponseError::PermissionDenied(
            format!("you do not own this mood entry. mood entry: {}", mood_id)
        ));
    }

    get_mood_field_via_id(conn, initiator, result[0].get(0)).await
}

/**
 * GET /entries
 * returns the root html if requesting html. otherwise will send back a list of
 * available and allowed entries for the current user from the session
 */
pub async fn handle_get_entries(
    req: HttpRequest, 
    session: Session,
    app: web::Data<state::AppState>,
    info: web::Query<json::QueryEntries>,
) -> app_error::Result<impl Responder> {
    let conn = &*app.get_conn().await?;
    let accept_html = response::check_if_html_req(&req, true).unwrap();
    let initiator_opt = from::get_initiator(conn, &session).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(Some(initiator_opt.unwrap().user)))
        } else {
            Ok(response::redirect_to_path("/auth/login"))
        }
    } else if initiator_opt.is_none() {
        Err(app_error::ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();

        Ok(response::json::respond_json(
            http::StatusCode::OK, 
            response::json::MessageDataJSON::build(
                "successful",
                json::search_entries(conn, json::SearchEntriesOptions {
                    owner: initiator.user.get_id(),
                    from: info.from,
                    to: info.to,
                    is_private: None
                }).await?
            )
        ))
    }
}

/**
 * POST /entries
 * creates a new entry when given a date for the current user from the session.
 * will also create text and mood entries if given as well
 */
pub async fn handle_post_entries(
    initiator: from::Initiator,
    app_data: web::Data<state::AppState>,
    posted: web::Json<PostEntryJson>
) -> app_error::Result<impl Responder> {
    let app = app_data.into_inner();
    let conn = &mut *app.get_conn().await?;
    let created = match &posted.created {
        Some(s) => s.clone(),
        None => chrono::Utc::now()
    };

    let entry_check = conn.query(
        "select id from entries where day = $1 and owner = $2",
        &[&created, &initiator.user.get_id()]
    ).await?;

    if entry_check.len() != 0 {
        return Err(app_error::ResponseError::EntryExists(
            format!("{}", created)
        ));
    }

    let transaction = conn.transaction().await?;
    let result = transaction.query_one(
        "insert into entries (day, owner) values ($1, $2) returning id, day, owner",
        &[&created, &initiator.user.get_id_ref()]
    ).await?;
    let entry_id: i32 = result.get(0);

    let mut mood_entries: Vec<json::MoodEntryJson> = vec!();

    if let Some(m) = &posted.mood_entries {
        for mood_entry in m {
            let field = get_mood_field_via_id(&transaction, initiator.user.get_id(), mood_entry.field_id).await?;

            db::mood_fields::verifiy(&field.get_config(), &mood_entry.value)?;

            let value_json = serde_json::to_value(mood_entry.value.clone())?;
            let result = transaction.query_one(
                r#"
                insert into mood_entries (field, value, comment, entry) values
                ($1, $2, $3, $4)
                returning id
                "#,
                &[&field.get_id(), &value_json, &mood_entry.comment, &entry_id]
            ).await?;

            mood_entries.push(json::MoodEntryJson {
                id: result.get(0),
                field: field.get_name(),
                field_id: field.get_id(),
                value: mood_entry.value.clone(),
                comment: clone_string_option(&mood_entry.comment),
                entry: entry_id
            });
        }
    }

    let mut text_entries: Vec<json::TextEntryJson> = vec!();

    if let Some(t) = &posted.text_entries {
        for text_entry in t {
            let result = transaction.query_one(
                "insert into text_entries (thought, private, entry) values ($1, $2, $3) returning id, thought, private",
                &[&text_entry.thought, &text_entry.private, &entry_id]
            ).await?;

            text_entries.push(json::TextEntryJson {
                id: result.get(0),
                thought: result.get(1),
                entry: entry_id,
                private: result.get(2)
            });
        }
    }

    transaction.commit().await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful", 
            json::EntryJson {
                id: result.get(0),
                created: result.get(1),
                owner: initiator.user.get_id(),
                mood_entries,
                text_entries
            }
        )
    ))
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
pub async fn handle_get_entries_id(
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
            Ok(response::redirect_to_path("/auth/login"))
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
pub async fn handle_put_entries_id(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    path: web::Path<EntryPath>,
    posted: web::Json<PutEntryJson>
) -> app_error::Result<impl Responder> {
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
        mood_entries: vec!(),
        text_entries: vec!(),
        owner: initiator.user.get_id()
    };
    let entry_id_list = vec!(path.entry_id);

    if let Some(m) = &posted.mood_entries {
        let mut ids: Vec<i32> = vec!();
        let mut mood_entries: Vec<json::MoodEntryJson> = vec!();

        for mood_entry in m {
            if let Some(id) = mood_entry.id {
                let field = get_mood_field_via_mood_entry(&transaction, initiator.user.get_id(), id).await?;

                db::mood_fields::verifiy(&field.get_config(), &mood_entry.value)?;
            
                let value_json = serde_json::to_value(mood_entry.value.clone())?;
                let _result = transaction.execute(
                    r#"
                    update mood_entries
                    set value = $1,
                        comment = $2
                    where id = $3
                    "#,
                    &[&value_json, &mood_entry.comment, &id]
                ).await?;

                ids.push(id);
                mood_entries.push(json::MoodEntryJson {
                    id: id,
                    field: field.get_name(),
                    field_id: field.get_id(),
                    value: mood_entry.value.clone(),
                    comment: clone_string_option(&mood_entry.comment),
                    entry: path.entry_id
                });
            } else {
                let field_id = match mood_entry.field_id {
                    Some(id) => id,
                    None => Err(app_error::ResponseError::Validation(
                        "no mood entry id was specified to update".to_owned()
                    ))?
                };

                let field = get_mood_field_via_id(&transaction, initiator.user.get_id(), field_id).await?;

                db::mood_fields::verifiy(&field.get_config(), &mood_entry.value)?;

                let value_json = serde_json::to_value(mood_entry.value.clone())?;
                let result = transaction.query_one(
                    r#"
                    insert into mood_entries (field, value, comment, entry) values
                    ($1, $2, $3, $4)
                    returning id
                    "#,
                    &[&field_id, &value_json, &mood_entry.comment, &path.entry_id]
                ).await?;

                ids.push(result.get(0));
                mood_entries.push(json::MoodEntryJson {
                    id: result.get(0),
                    field: field.get_name(),
                    field_id: field.get_id(),
                    value: mood_entry.value.clone(),
                    comment: clone_string_option(&mood_entry.comment),
                    entry: path.entry_id
                });
            }
        }

        rtn.mood_entries.append(&mut mood_entries);

        let left_over = transaction.query(
            "select id from mood_entries where entry = $1 and not (id = any($2))",
            &[&path.entry_id, &ids]
        ).await?;

        if left_over.len() > 0 {
            let mut to_delete = Vec::<i32>::with_capacity(left_over.len());

            for row in left_over {
                to_delete.push(row.get(0));
            }

            let _result = transaction.execute(
                "delete from mood_entries where id = any($1)",
                &[&to_delete]
            ).await?;
        }
    } else {
        rtn.mood_entries = json::search_mood_entries(&transaction, &entry_id_list).await?;
    }

    if let Some(t) = &posted.text_entries {
        let mut ids: Vec<i32> = vec!();
        let mut text_entries: Vec<json::TextEntryJson> = vec!();

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

                if check[0].get::<usize, i32>(0) != initiator.user.get_id() {
                    return Err(app_error::ResponseError::PermissionDenied(
                        format!("you do not have permission to modify another users text entry. text entry: {}", id)
                    ));
                }

                let result = transaction.query_one(
                    "update text_entries set thought = $1, private = $2 where id = $3 returning id, thought, private",
                    &[&text_entry.thought, &text_entry.private, &id]
                ).await?;

                ids.push(id);
                text_entries.push(json::TextEntryJson {
                    id: result.get(0),
                    thought: result.get(1),
                    entry: path.entry_id,
                    private: result.get(2)
                });
            } else {
                let result = transaction.query_one(
                    "insert into text_entries (thought, private, entry) values ($1, $2, $3) returning id, thought, private",
                    &[&text_entry.thought, &text_entry.private, &path.entry_id]
                ).await?;

                ids.push(result.get(0));
                text_entries.push(json::TextEntryJson {
                    id: result.get(0),
                    thought: result.get(1),
                    entry: path.entry_id,
                    private: result.get(2)
                })
            }
        }

        rtn.text_entries.append(&mut text_entries);

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
pub async fn handle_delete_entries_id(
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

    let _mood_result = transaction.execute(
        "delete from mood_entries where entry = $1",
        &[&path.entry_id]
    ).await?;

    let _entry_result = transaction.execute(
        "delete from entries where id = $1",
        &[&path.entry_id]
    ).await?;

    transaction.commit().await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::<Option<()>>::build(
            "successful",
            None
        )
    ))
}