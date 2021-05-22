use actix_web::{web, http, HttpRequest, Responder};
use actix_session::{Session};

pub mod entry_id;

use crate::error;
use crate::request::from;
use crate::response;
use crate::state;
use crate::json;
use crate::parsing::url_paths;
use crate::security;

pub async fn handle_get(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>,
    path: web::Path<url_paths::UserPath>,
    info: web::Query<json::QueryEntries>,
) -> error::Result<impl Responder> {
    let accept_html = response::check_if_html_req(&req, true)?;
    let conn = &*app.get_conn().await?;
    let initiator_opt = from::get_initiator(conn, &session).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(Some(initiator_opt.unwrap().user)))
        } else {
            let redirect = format!("/auth/login?jump_to=/users/{}/entries", path.user_id);
            Ok(response::redirect_to_path(redirect.as_str()))
        }
    } else if initiator_opt.is_none() {
        Err(error::ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();
        security::assert::permission_to_read(conn, initiator.user.get_id(), path.user_id).await?;

        Ok(response::json::respond_json(
            http::StatusCode::OK,
            response::json::MessageDataJSON::build(
                "successful",
                json::search_entries(conn, json::SearchEntriesOptions {
                    owner: path.user_id,
                    from: info.from,
                    to: info.to,
                    is_private: Some(false)
                }).await?
            )
        ))
    }
}