use actix_web::{web, http, HttpRequest, Responder};
use actix_session::{Session};
use serde::{Serialize};

use crate::error;
use crate::response;
use crate::request::from;
use crate::state;



pub async fn handle_get_backup(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>
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
            response::json::MessageDataJSON::<Option<()>>::build(
                "successful",
                None
            )
        ))
    }
}

pub async fn handle_post_backup(
    initiator: from::Initiator,
    app: web::Data<state::AppState>
) -> error::Result<impl Responder> {
    Ok(response::json::respond_okay())
}