use actix_web::{web, http, HttpRequest, HttpResponse, Responder, error};
use actix_session::{Session};
use serde::{Deserialize};

use crate::db;
use crate::response;
use crate::request::from;
use crate::error as app_error;

pub mod auth;
pub mod entries;
pub mod mood_fields;
pub mod users;
pub mod text_entries;
pub mod mood_entries;

pub async fn handle_get_dashboard(session: Session) -> app_error::Result<impl Responder> {
    let check = session.get::<String>("token")?;

    if check.is_some() {
        Ok(response::respond_index_html())
    } else {
        Ok(HttpResponse::Found().insert_header((http::header::LOCATION, "/auth/login")).finish())
    }
}

pub async fn handle_get_root(session: Session) -> app_error::Result<impl Responder> {
    let check = session.get::<String>("token")?;

    if check.is_some() {
        Ok(HttpResponse::Found().insert_header((http::header::LOCATION, "/dashboard")).finish())
    } else {
        Ok(HttpResponse::Found().insert_header((http::header::LOCATION, "/auth/login")).finish())
    }
}

pub async fn okay() -> impl Responder {
    HttpResponse::Ok().body("okay")
}

#[derive(Deserialize)]
pub struct StaticPath {
    file_path: String
}

pub async fn handle_get_static(
    path: web::Path<StaticPath>
) -> actix_web::Result<impl Responder> {
    let mut file_path = std::env::current_dir()?;
    file_path.push("static");

    let file_handle = std::fs::File::open(file_path)?;
    
    Ok(actix_files::NamedFile::from_file(file_handle, path.file_path.clone()))
}

pub fn handle_json_error(
    err: error::JsonPayloadError,
    _req: &HttpRequest
) -> error::Error {
    let err_str = err.to_string();
    error::InternalError::from_response(
        err, 
        response::json::respond_json(
            http::StatusCode::CONFLICT,
            response::json::ErrorJSON::build_with_err(
                "given json is not valid",
                "invalid json",
                err_str
            )
        )
    ).into()
}