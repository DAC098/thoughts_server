use actix_web::{web, http, HttpRequest, Responder};
use actix_session::{Session};

pub mod entries;
pub mod mood_fields;
pub mod tags;

use crate::error;
use crate::request::from;
use crate::response;
use crate::state;
use crate::parsing::url_paths;
use crate::security;
use crate::db;

pub async fn handle_get(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>,
    path: web::Path<url_paths::UserPath>,
) -> error::Result<impl Responder> {
    let accept_html = response::check_if_html_req(&req, true)?;
    let conn = &*app.get_conn().await?;
    let initiator_opt = from::get_initiator(conn, &session).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(Some(initiator_opt.unwrap().user)))
        } else {
            let redirect = format!("/auth/login?jump_to=/users/{}", path.user_id);
            Ok(response::redirect_to_path(redirect.as_str()))
        }
    } else if initiator_opt.is_none() {
        Err(error::ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();
        security::assert::permission_to_read(conn, initiator.user.get_id(), path.user_id).await?;
        let user_opt = db::users::get_via_id(conn, path.user_id).await?;

        Ok(response::json::respond_json(
            http::StatusCode::OK,
            response::json::MessageDataJSON::build(
                "successful",
                user_opt.unwrap()
            )
        ))
    }
}