use actix_web::{web, http, Responder};
use actix_session::{Session};
use serde::{Deserialize};

use tlib::db::{users, user_sessions};

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
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
    session: Session
) -> error::Result<impl Responder> {
    let conn = &*db.get_conn().await?;

    match from::get_initiator(conn, &session).await? {
        Some(_) => Ok(response::redirect_to_path("/entries")),
        None => Ok(response::respond_index_html(&template.into_inner(), None)?)
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
    session: Session,
    db: state::WebDbState,
    posted: web::Json<LoginBodyJSON>
) -> error::Result<impl Responder> {
    let conn = &mut *db.get_conn().await?;
    let posted = posted.into_inner();
    let result = conn.query_opt(
        "select id, hash from users where username = $1 or email = $1",
        &[&posted.username]
    ).await?;

    if result.is_none() {
        return Err(error::ResponseError::UsernameNotFound(posted.username.clone()));
    }
    
    let row = result.unwrap();

    security::verify_password(row.get(1), &posted.password)?;

    let transaction = conn.transaction().await?;
    let token = uuid::Uuid::new_v4();

    user_sessions::insert(&transaction, token, row.get(0)).await?;

    session.insert("token", token)?;

    transaction.commit().await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "login successful",
            users::find_from_id(conn, &row.get(0)).await?
        )
    ))
}