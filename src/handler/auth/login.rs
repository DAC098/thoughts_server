use actix_web::{web, http, Responder};
use actix_session::{Session};
use serde::{Deserialize};

use crate::db::users;
use crate::db::user_sessions;
use crate::response;
use crate::state;
use crate::request::from;
use crate::security;

use response::error;

/**
 * GET /auth/login
 * currently will only serve the html
 */
pub async fn handle_get(
    app: web::Data<state::AppState>,
    session: Session
) -> error::Result<impl Responder> {
    let app = app.into_inner();
    let conn = app.get_conn().await?;

    match from::get_initiator(&conn, &session).await? {
        Some(_) => Ok(response::redirect_to_path("/entries")),
        None => Ok(response::respond_index_html(None))
    }
}

#[derive(Deserialize)]
pub struct LoginBodyJSON {
    username: String,
    password: String
}

/**
 * POST /auth/login
 * receives the login information from a user request. if accepted then it will
 * send back a successful message with a login session
 */
pub async fn handle_post(
    app: web::Data<state::AppState>,
    session: Session, 
    posted: web::Json<LoginBodyJSON>
) -> error::Result<impl Responder> {
    let conn = &mut *app.get_conn().await?;
    let result = conn.query(
        "select id, hash from users where username = $1 or email = $1",
        &[&posted.username]
    ).await?;

    if result.len() == 0 {
        return Err(error::ResponseError::UsernameNotFound(posted.username.clone()));
    }

    security::verify_password(result[0].get(1), &posted.password)?;

    let transaction = conn.transaction().await?;
    let token = uuid::Uuid::new_v4();

    user_sessions::insert(&transaction, token, result[0].get(0)).await?;

    session.insert("token", token)?;

    transaction.commit().await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "login successful",
            users::get_via_id(conn, result[0].get(0)).await?
        )
    ))
}