use actix_web::{web, http, Responder};
use actix_session::{Session};
use serde::{Deserialize};
use argon2::{Config, ThreadMode, Variant, Version};

use crate::db::users;
use crate::db::user_sessions;
use crate::error;
use crate::response;
use crate::state;
use crate::request::from;

fn default_argon2_config() -> Config<'static> {
    Config {
        variant: Variant::Argon2i,
        version: Version::Version13,
        mem_cost: 65536,
        time_cost: 10,
        lanes: 4,
        thread_mode: ThreadMode::Parallel,
        secret: &[],
        ad: &[],
        hash_length: 32
    }
}

fn generate_new_hash_with_config(
    password: &String, 
    config: &Config
) -> error::Result<String> {
    let mut salt: [u8; 64] = [0; 64];
    openssl::rand::rand_bytes(&mut salt)?;

    Ok(argon2::hash_encoded(
        &password.as_bytes(), 
        &salt,
        &config
    )?)
}

fn generate_new_hash(password: &String) -> error::Result<String> {
    let config = default_argon2_config();

    generate_new_hash_with_config(
        password, 
        &config
    )
}

fn verify_password(hash: &str, password: &String) -> error::Result<()> {
    let matches = argon2::verify_encoded(hash, password.as_bytes())?;

    if !matches {
        Err(error::ResponseError::InvalidPassword)
    } else {
        Ok(())
    }
}

/**
 * GET /auth/login
 * currently will only serve the html
 */
pub async fn handle_get_auth_login(
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
pub async fn handle_post_auth_login(
    app: web::Data<state::AppState>,
    session: Session, 
    posted: web::Json<LoginBodyJSON>
) -> error::Result<impl Responder> {
    let conn = &mut app.get_conn().await?;
    let result = conn.query(
        "select id, hash from users where username = $1",
        &[&posted.username]
    ).await?;

    if result.len() == 0 {
        return Err(error::ResponseError::UsernameNotFound(posted.username.clone()));
    }

    verify_password(result[0].get(1), &posted.password)?;

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

pub async fn handle_post_auth_logout(
    session: Session,
    app: web::Data<state::AppState>,
) -> error::Result<impl Responder> {
    let conn = &mut app.get_conn().await?;
    let token_opt = from::get_session_token(&session)?;
    session.purge();
    
    if let Some(token) = token_opt {
        let transaction = conn.transaction().await?;
        user_sessions::delete(&transaction, token).await?;

        transaction.commit().await?;
    }

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::<Option<()>>::build(
            "logout successful",
            None
        )
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
    let conn = &mut app.get_conn().await?;
    let (found_username, found_email) = users::check_username_email(
        conn, &posted.username, &posted.email
    ).await?;

    if found_username {
        return Err(error::ResponseError::UsernameExists(posted.username.clone()));
    }

    if found_email {
        return Err(error::ResponseError::EmailExists(posted.email.as_ref().unwrap().clone()))
    }

    let hash = generate_new_hash(&posted.password)?;
    let transaction = conn.transaction().await?;
    let user = users::insert(
        &transaction, 
        &posted.username,
        &hash,
        &posted.email
    ).await?;
    let token = uuid::Uuid::new_v4();

    user_sessions::insert(&transaction, token, user.get_id()).await?;

    session.insert("token", token)?;

    transaction.commit().await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::<Option<()>>::build("created account", None)
    ))
}

#[derive(Deserialize)]
pub struct ChangePasswordJson {
    current_password: String,
    new_password: String
}

pub async fn handle_post_auth_change(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    posted: web::Json<ChangePasswordJson>,
) -> error::Result<impl Responder> {
    let conn = &mut app.get_conn().await?;
    let result = conn.query_one(
        "select id, hash from users where id = $1",
        &[&initiator.user.get_id()]
    ).await?;

    verify_password(result.get(1), &posted.current_password)?;

    let hash = generate_new_hash(&posted.new_password)?;
    let transaction = conn.transaction().await?;
    let _insert_result = transaction.execute(
        "update users set hash = $1 where id = $2",
        &[&hash, &initiator.user.get_id()]
    ).await?;

    transaction.commit().await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::<Option<()>>::build("password changed", None)
    ))
}