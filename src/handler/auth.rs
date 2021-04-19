use actix_web::{web, http, HttpResponse, Responder};
use actix_session::{Session};
use serde::{Deserialize};
use argon2::{Config, ThreadMode, Variant, Version};

use crate::db::users;
use crate::db::user_sessions;
use crate::error;
use crate::response;
use crate::state;

/**
 * GET /auth/login
 * currently will only serve the html
 */
pub async fn handle_get_auth_login(session: Session) -> error::Result<impl Responder> {
    let check = session.get::<String>("token")?;

    if check.is_some() {
        Ok(HttpResponse::Found().insert_header((http::header::LOCATION, "/entries")).finish())
    } else {
        Ok(response::respond_index_html())
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
pub async fn handle_post_auth_login(
    app: web::Data<state::AppState>,
    session: Session, 
    posted: web::Json<LoginBodyJSON>
) -> error::Result<impl Responder> {
    let conn = &app.get_conn().await?;
    let result = conn.query(
        "select id, hash from users where username = $1",
        &[&posted.username]
    ).await?;

    if result.len() == 0 {
        return Err(error::ResponseError::UsernameNotFound(posted.username.clone()));
    }

    let matches = argon2::verify_encoded(result[0].get(1), posted.password.as_bytes())?;

    if !matches {
        return Err(error::ResponseError::InvalidPassword);
    }

    let token = uuid::Uuid::new_v4();
    user_sessions::UserSession::insert(conn, token, result[0].get(0)).await?;

    session.insert("token", token)?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::<Option<()>>::build("login successful", None)
    ))
}

#[derive(Deserialize)]
pub struct NewLoginJSON {
    username: String,
    password: String,
    email: Option<String>
}

/**
 * POST /auth/create
 */
pub async fn handle_post_auth_create(
    app: web::Data<state::AppState>,
    session: Session,
    posted: web::Json<NewLoginJSON>
) -> error::Result<impl Responder> {
    let conn = &app.get_conn().await?;
    let (found_username, found_email) = users::User::check_username_email(
        conn, &posted.username, &posted.email
    ).await?;

    if found_username {
        return Err(error::ResponseError::UsernameExists(posted.username.clone()));
    }

    if found_email {
        return Err(error::ResponseError::EmailExists(posted.email.as_ref().unwrap().clone()))
    }

    let config = Config {
        variant: Variant::Argon2i,
        version: Version::Version13,
        mem_cost: 65536,
        time_cost: 10,
        lanes: 4,
        thread_mode: ThreadMode::Parallel,
        secret: &[],
        ad: &[],
        hash_length: 32
    };

    let mut salt: [u8; 64] = [0; 64];
    openssl::rand::rand_bytes(&mut salt)?;

    let hash = argon2::hash_encoded(
        &posted.password.as_bytes(), 
        &salt, 
        &config
    )?;

    let user = users::User::insert(
        conn, 
        &posted.username,
        &hash,
        &posted.email
    ).await?;

    let token = uuid::Uuid::new_v4();
    user_sessions::UserSession::insert(conn, token, user.get_id()).await?;

    session.insert("token", token)?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::<Option<()>>::build("created account", None)
    ))
}