use std::fmt::{Write};

use actix_web::{web, http, HttpRequest, Responder};
use actix_session::{Session};
use serde::{Deserialize};

pub mod user_id;

use crate::error;
use crate::request::from;
use crate::response;
use crate::state;
use crate::security;
use crate::db;
use crate::json;

#[derive(Deserialize)]
pub struct UserSearchQuery {
    level: Option<i32>,
    full_name: Option<String>,
    username: Option<String>
}

pub async fn handle_get(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>,
    info: web::Query<UserSearchQuery>,
) -> error::Result<impl Responder> {
    let accept_html = response::check_if_html_req(&req, true)?;
    let conn = &*app.get_conn().await?;
    let initiator_opt = from::get_initiator(conn, &session).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(Some(initiator_opt.unwrap().user)))
        } else {
            Ok(response::redirect_to_path("/auth/login?jump_to=/admin/users"))
        }
    } else if initiator_opt.is_none() {
        Err(error::ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();

        if initiator.user.level != 1 {
            Err(error::ResponseError::PermissionDenied(
                format!("you do not have permission to view all users")
            ))
        } else {
            let mut arg_count: usize = 2;
            let mut query_str = "select id, username, level, full_name, email, email_verified from users where id != $1".to_owned();
            let mut query_slice: Vec<&(dyn tokio_postgres::types::ToSql + std::marker::Sync)> = vec![&initiator.user.id];

            if let Some(level) = info.level.as_ref() {
                write!(&mut query_str, " and level = ${}", arg_count)?;
                query_slice.push(level);
                arg_count += 1;
            }

            if let Some(full_name) = info.full_name.as_ref() {
                write!(&mut query_str, " and full_name ilike ${}", arg_count)?;
                query_slice.push(full_name);
                arg_count += 1;
            }

            if let Some(username) = info.username.as_ref() {
                write!(&mut query_str, " and username ilike ${}", arg_count)?;
                query_slice.push(username);
            }

            let result = conn.query(query_str.as_str(), &query_slice[..]).await?;
            let mut rtn: Vec<db::users::User> = Vec::with_capacity(result.len());

            for row in result {
                rtn.push(db::users::User {
                    id: row.get(0),
                    username: row.get(1),
                    level: row.get(2),
                    full_name: row.get(3),
                    email: row.get(4),
                    email_verified: row.get(5)
                });
            }

            Ok(response::json::respond_json(
                http::StatusCode::OK,
                response::json::MessageDataJSON::build(
                    "successful", rtn
                )
            ))
        }
    }
}

#[derive(Deserialize)]
pub struct PostUserJson {
    username: String,
    password: String,
    email: String,
    full_name: Option<String>,
    level: i32
}

pub async fn handle_post(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    posted: web::Json<PostUserJson>,
) -> error::Result<impl Responder> {
    if initiator.user.level != 1 {
        return Err(error::ResponseError::PermissionDenied(
            format!("you do not have permission to create new users")
        ));
    }

    let conn = &mut *app.get_conn().await?;
    let (found_username, found_email) = db::users::check_username_email(
        conn, &posted.username, &posted.email
    ).await?;

    if found_username {
        return Err(error::ResponseError::UsernameExists(posted.username.clone()));
    }

    if found_email {
        return Err(error::ResponseError::EmailExists(posted.email.clone()))
    }

    let hash = security::generate_new_hash(&posted.password)?;
    let transaction = conn.transaction().await?;

    let result = transaction.query_one(
        r#"
        insert into users (level, username, full_name, hash, email) 
        values ($1, $2, $3, $4, $5)
        returning id
        "#,
        &[&posted.level, &posted.username, &posted.full_name, &hash, &posted.email]
    ).await?;

    transaction.commit().await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "created account",
            json::UserInfoJson {
                id: result.get(0),
                username: posted.username.clone(),
                level: posted.level,
                full_name: posted.full_name.clone(),
                email: posted.email.clone(),
                user_access: vec!()
            }
        )
    ))
}