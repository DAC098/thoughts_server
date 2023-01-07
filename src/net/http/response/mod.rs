use actix_web::{http, HttpRequest, HttpResponse, Responder};
use serde_json::json;

use crate::db::tables::users;

pub mod json;

use crate::state::TemplateState;
use super::error;

pub fn respond_index_html(
    template_state: &TemplateState,
    user_opt: Option<users::User>
) -> error::Result<HttpResponse> {
    let render_data = json!({"user": user_opt});
    let mut builder = HttpResponse::build(http::StatusCode::OK);
    builder.insert_header((http::header::CONTENT_TYPE, "text/html"));

    Ok(builder.body(
        template_state.render("pages/index", &render_data)?
    ))
}

pub fn check_if_html(
    headers: &http::header::HeaderMap
) -> Result<bool, http::header::ToStrError> {
    let accept_opt = headers.get("accept");

    if let Some(accept_type) = accept_opt {
        match accept_type.to_str() {
            Ok(accept) => Ok(accept.contains("text/html")),
            Err(e) => Err(e)
        }
    } else {
        Ok(false)
    }
}

pub fn try_check_if_html(
    headers: &http::header::HeaderMap,
) -> bool {
    if let Ok(answer) = check_if_html(headers) {
        answer
    } else {
        false
    }
}

#[inline]
pub fn try_check_if_html_req(
    req: &HttpRequest
) -> bool {
    try_check_if_html(req.headers())
}

pub fn redirect_to_path(path: &str) -> HttpResponse {
    HttpResponse::Found().insert_header((http::header::LOCATION, path)).finish()
}

pub fn redirect_to_login(req: &HttpRequest) -> HttpResponse {
    let redirect_path = format!("/auth/session?jump_to={}", urlencoding::encode(
        if let Some(path_and_query) = req.uri().path_and_query() {
            path_and_query.as_str()
        } else {
            req.uri().path()
        }
    ));

    HttpResponse::Found().insert_header((http::header::LOCATION, redirect_path.as_str())).finish()
}

pub fn redirect_to_login_with(path: &str) -> HttpResponse {
    let redirect_path = format!("/auth/session?jump_to={}", urlencoding::encode(path));

    HttpResponse::Found().insert_header((http::header::LOCATION, redirect_path.as_str())).finish()
}

#[inline]
pub fn okay_response() -> HttpResponse {
    HttpResponse::Ok()
        .insert_header((http::header::CONTENT_TYPE, "text/plain"))
        .body("okay")
}

#[allow(dead_code)]
pub async fn okay() -> impl Responder {
    okay_response()
}