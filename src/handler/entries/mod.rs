use std::collections::{HashMap};

use actix_web::{web, http, HttpRequest, Responder};
use actix_session::{Session};
use serde::{Deserialize};

pub mod entry_id;

use crate::db;
use crate::response;
use crate::state;
use crate::request::from;
use crate::json;
use crate::util;

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
pub struct PostEntryJson {
    created: chrono::DateTime<chrono::Utc>,
    tags: Option<Vec<i32>>,
    custom_field_entries: Option<Vec<PostCustomFieldEntryJson>>,
    text_entries: Option<Vec<PostTextEntryJson>>
}

/**
 * GET /entries
 * returns the root html if requesting html. otherwise will send back a list of
 * available and allowed entries for the current user from the session
 */
pub async fn handle_get(
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
            Ok(response::redirect_to_path("/auth/login?jump_to=/entries"))
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
pub async fn handle_post(
    initiator: from::Initiator,
    app_data: web::Data<state::AppState>,
    posted: web::Json<PostEntryJson>
) -> app_error::Result<impl Responder> {
    let app = app_data.into_inner();
    let conn = &mut *app.get_conn().await?;

    let entry_check = conn.query(
        "select id from entries where day = $1 and owner = $2",
        &[&posted.created, &initiator.user.get_id()]
    ).await?;

    if entry_check.len() != 0 {
        return Err(app_error::ResponseError::EntryExists(
            format!("{}", posted.created)
        ));
    }

    let transaction = conn.transaction().await?;
    let result = transaction.query_one(
        "insert into entries (day, owner) values ($1, $2) returning id, day, owner",
        &[&posted.created, &initiator.user.get_id_ref()]
    ).await?;
    let entry_id: i32 = result.get(0);

    let mut custom_field_entries: HashMap<i32, json::CustomFieldEntryJson> = HashMap::new();

    if let Some(m) = &posted.custom_field_entries {
        for custom_field_entry in m {
            let field = db::custom_fields::get_via_id(&transaction, custom_field_entry.field, Some(initiator.user.id)).await?;

            db::custom_fields::verifiy(&field.config, &custom_field_entry.value)?;

            let value_json = serde_json::to_value(custom_field_entry.value.clone())?;
            let _result = transaction.execute(
                r#"
                insert into custom_field_entries (field, value, comment, entry) values
                ($1, $2, $3, $4)
                "#,
                &[&field.id, &value_json, &custom_field_entry.comment, &entry_id]
            ).await?;

            custom_field_entries.insert(field.id, json::CustomFieldEntryJson {
                field: field.id,
                name: field.name,
                value: custom_field_entry.value.clone(),
                comment: util::clone_option(&custom_field_entry.comment),
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

    let mut entry_tags: Vec<i32> = vec!();

    if let Some(tags) = &posted.tags {
        for tag_id in tags {
            let _result = transaction.execute(
                "insert into entries2tags (tag, entry) values ($1, $2)",
                &[&tag_id, &entry_id]
            ).await?;

            entry_tags.push(*tag_id);
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
                tags: entry_tags,
                custom_field_entries,
                text_entries
            }
        )
    ))
}