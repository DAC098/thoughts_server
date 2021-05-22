use actix_web::{web, http, HttpRequest, Responder};
use actix_session::{Session};
use serde::{Deserialize};

use crate::error::{Result, ResponseError};
use crate::request::from;
use crate::response;
use crate::state;
use crate::json;
use crate::db;
use crate::security;

#[derive(Deserialize)]
pub struct MoodFieldPath {
    field_id: i32
}

pub async fn handle_get(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>,
    path: web::Path<MoodFieldPath>
) -> Result<impl Responder> {
    let accept_html = response::check_if_html_req(&req, true)?;
    let conn = &*app.get_conn().await?;
    let initiator_opt = from::get_initiator(conn, &session).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(Some(initiator_opt.unwrap().user)))
        } else {
            let redirect = format!("/auth/login?jump_to=/mood_fields/{}", path.field_id);
            Ok(response::redirect_to_path(redirect.as_str()))
        }
    } else if initiator_opt.is_none() {
        Err(ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();

        if let Some(field) = json::search_mood_field(conn, path.field_id).await? {
            if field.owner == initiator.user.get_id() {
                Ok(response::json::respond_json(
                    http::StatusCode::OK,
                    response::json::MessageDataJSON::build(
                        "successful",
                        field
                    )
                ))
            } else {
                Err(ResponseError::PermissionDenied(
                    format!("you do not have permission to view this users mood field as you are not the owner")
                ))
            }
        } else {
            Err(ResponseError::MoodFieldNotFound(path.field_id))
        }
    }
}

#[derive(Deserialize)]
pub struct PutMoodFieldJson {
    name: String,
    config: db::mood_fields::MoodFieldType,
    comment: Option<String>
}

pub async fn handle_put(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    path: web::Path<MoodFieldPath>,
    posted: web::Json<PutMoodFieldJson>,
) -> Result<impl Responder> {
    let conn = &*app.get_conn().await?;
    security::assert::is_owner_for_mood_field(conn, path.field_id, initiator.user.get_id()).await?;

    let config_json = serde_json::to_value(posted.config.clone())?;
    let result = conn.query_one(
        r#"
        update mood_fields
        set name = $1,
            config = $2,
            comment = $3
        where id = $4
        returning name, comment
        "#,
        &[
            &posted.name,
            &config_json,
            &posted.comment,
            &path.field_id
        ]
    ).await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful",
            json::MoodFieldJson {
                id: path.field_id,
                name: result.get(0),
                config: posted.config.clone(),
                comment: result.get(1),
                owner: initiator.user.get_id(),
                issued_by: None
            }
        )
    ))
}

pub async fn handle_delete(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    path: web::Path<MoodFieldPath>,
) -> Result<impl Responder> {
    let conn = &*app.get_conn().await?;
    security::assert::is_owner_for_mood_field(conn, path.field_id, initiator.user.get_id()).await?;

    let _mood_entries_result = conn.execute(
        "delete from mood_entries where field = $1",
        &[&path.field_id]
    ).await?;

    let _mood_field_result = conn.execute(
        "delete from mood_fields where id = $1",
        &[&path.field_id]
    ).await?;

    Ok(response::json::respond_okay())
}