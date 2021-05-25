use actix_web::{web, http, HttpRequest, Responder};
use actix_session::{Session};
use serde::{Serialize, Deserialize};

use crate::error;
use crate::response;
use crate::request::from;
use crate::state;
use crate::json;
use crate::db;

#[derive(Serialize, Deserialize)]
pub struct BackupJson {
    custom_fields: Vec<json::CustomFieldJson>,
    tags: Vec<db::tags::Tag>,
    entries: Vec<json::EntryJson>
}

pub async fn handle_get(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>,
    info: web::Query<json::QueryEntries>,
) -> error::Result<impl Responder> {
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
        Err(error::ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();

        Ok(response::json::respond_json(
            http::StatusCode::OK,
            BackupJson { 
                custom_fields: json::search_custom_fields(conn, initiator.user.id).await?, 
                tags: db::tags::get_via_owner(conn, initiator.user.id).await?, 
                entries: json::search_entries(conn, json::SearchEntriesOptions { 
                    from: info.from,
                    to: info.to,
                    owner: initiator.user.id,
                    is_private: None
                }).await?
            }
        ))
    }
}

pub async fn handle_post(
    _initiator: from::Initiator,
    _app: web::Data<state::AppState>
) -> error::Result<impl Responder> {
    Ok(response::json::respond_okay())
}