use actix_web::{web, http, HttpRequest, HttpResponse, Responder};
use actix_session::{Session};

use crate::error;
use crate::response;
use crate::state;
use crate::request::from;

pub async fn handle_get_users(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>,
) -> error::Result<impl Responder> {
    let accept_html = response::check_if_html_req(&req, true)?;
    let conn = &app.get_conn().await?;
    let initiator_opt = from::get_initiator(conn, session).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html())
        } else {
            Ok(response::redirect_to_path("/auth/login"))
        }
    } else if initiator_opt.is_none() {
        Err(error::ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();

        Ok(HttpResponse::Ok().body("okay"))
    }
}

pub async fn handle_get_users_id(req: HttpRequest) -> impl Responder {
    if response::check_if_html_req(&req, true).unwrap() {
        return response::respond_index_html();
    }

    return HttpResponse::Ok().body("okay");
}