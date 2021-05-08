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

pub async fn handle_get_mood_fields(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>,
) -> Result<impl Responder> {
    let accept_html = response::check_if_html_req(&req, true)?;
    let conn = &*app.get_conn().await?;
    let initiator_opt = from::get_initiator(conn, &session).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(Some(initiator_opt.unwrap().user)))
        } else {
            Ok(response::redirect_to_path("/auth/login"))
        }
    } else if initiator_opt.is_none() {
        Err(ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();

        Ok(response::json::respond_json(
            http::StatusCode::OK,
            response::json::MessageDataJSON::build(
                "successful",
                json::search_mood_fields(conn, initiator.user.get_id()).await?
            )
        ))
    }
    
}

#[derive(Deserialize)]
pub struct PostMoodFieldJson {
    name: String,
    config: db::mood_fields::MoodFieldType,
    comment: Option<String>
}

pub async fn handle_post_mood_fields(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    posted: web::Json<PostMoodFieldJson>,
) -> Result<impl Responder> {
    let conn = &*app.get_conn().await?;

    let check = conn.query(
        "select id from mood_fields where name = $1 and owner = $2",
        &[&posted.name, &initiator.user.get_id()]
    ).await?;

    if check.len() != 0 {
        return Err(ResponseError::MoodFieldExists(posted.name.clone()));
    }

    let config_json = serde_json::to_value(posted.config.clone())?;
    let result = conn.query_one(
        "insert into mood_fields (name, config, comment, owner) values 
        ($1, $2, $3, $4) 
        returning id, name, config, comment",
        &[
            &posted.name, 
            &config_json,
            &posted.comment, 
            &initiator.user.get_id()
        ]
    ).await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful",
            json::MoodFieldJson {
                id: result.get(0),
                name: result.get(1),
                config: serde_json::from_value(result.get(2))?,
                comment: result.get(3),
                owner: initiator.user.get_id(),
                issued_by: None
            }
        )
    ))
}

#[derive(Deserialize)]
pub struct MoodFieldPath {
    field_id: i32
}

pub async fn handle_get_mood_fields_id(
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
            Ok(response::redirect_to_path("/auth/login"))
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

pub async fn handle_put_mood_fields_id(
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

pub async fn handle_delete_mood_fields_id(
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

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::<Option<()>>::build(
            "successful",
            None
        )
    ))
}