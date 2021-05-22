use actix_web::{web, http, HttpRequest, Responder};
use actix_session::{Session};
use serde::{Deserialize};

pub mod field_id;

use crate::error::{Result, ResponseError};
use crate::request::from;
use crate::response;
use crate::state;
use crate::json;
use crate::db;

pub async fn handle_get(
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
            Ok(response::redirect_to_path("/auth/login?jump_to=/mood_fields"))
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

pub async fn handle_post(
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