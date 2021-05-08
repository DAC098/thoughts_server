use actix_web::{web, http, HttpRequest, Responder};
use actix_session::{Session};
use serde::{Deserialize};

use crate::error;
use crate::request::from;
use crate::response;
use crate::state;
use crate::db;

#[derive(Deserialize)]
pub struct UserIdPath {
    user_id: i32
}

pub async fn handle_get(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>,
    path: web::Path<UserIdPath>
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

        if initiator.user.level != 1 {
            Err(error::ResponseError::PermissionDenied(
                format!("you do not have permission to view this user information")
            ))
        } else {
            let result = conn.query(
                "select id, username, level, full_name, email from users where id = $1",
                &[&path.user_id]
            ).await?;

            if result.len() == 0 {
                Err(error::ResponseError::UserIDNotFound(path.user_id))
            } else {
                Ok(response::json::respond_json(
                    http::StatusCode::OK,
                    response::json::MessageDataJSON::build(
                        "successful",
                        db::users::User {
                            id: result[0].get(0),
                            username: result[0].get(1),
                            level: result[0].get(2),
                            full_name: result[0].get(3),
                            email: result[0].get(4)
                        }
                    )
                ))
            }
        }
    }
}

#[derive(Deserialize)]
pub struct PutUserJson {
    username: Option<String>,
    level: Option<i32>,
    full_name: Option<String>,
    email: Option<String>
}

pub async fn handle_put(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    posted: web::Json<PutUserJson>,
    path: web::Path<UserIdPath>,
) -> error::Result<impl Responder> {
    if initiator.user.level != 1 {
        return Err(error::ResponseError::PermissionDenied(
            format!("you do not have permission to alter another users information")
        ));
    }

    let conn = &mut *app.get_conn().await?;
    let mut arg_count: u32 = 2;
    let mut set_fields: Vec<String> = vec!();
    let mut query_slice: Vec<&(dyn tokio_postgres::types::ToSql + std::marker::Sync)> = vec!(&path.user_id);

    if let Some(username) = posted.username.as_ref() {
        set_fields.push(format!("username = ${}", arg_count));
        arg_count += 1;
        query_slice.push(username);
    }

    if let Some(level) = posted.level.as_ref() {
        set_fields.push(format!("level = ${}", arg_count));
        arg_count += 1;
        query_slice.push(level);
    }

    if let Some(full_name) = posted.full_name.as_ref() {
        set_fields.push(format!("full_name = ${}", arg_count));
        arg_count += 1;
        query_slice.push(full_name);
    }

    if let Some(email) = posted.email.as_ref() {
        set_fields.push(format!("email = ${}", arg_count));
        query_slice.push(email);
    }

    let query_str = format!(r#"
        update users
        set {}
        where id = $1
        returning id, username, level, full_name, email
    "#, set_fields.join(", "));

    let transaction = conn.transaction().await?;

    let result = transaction.query_one(query_str.as_str(), &query_slice[..]).await?;

    transaction.commit().await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful",
            db::users::User {
                id: path.user_id,
                username: result.get(1),
                level: result.get(2),
                full_name: result.get(3),
                email: result.get(4)
            }
        )
    ))
}

pub async fn handle_delete(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    path: web::Path<UserIdPath>,
) -> error::Result<impl Responder> {
    if initiator.user.level != 1 {
        return Err(error::ResponseError::PermissionDenied(
            format!("you do not have permission to delete another user")
        ));
    }

    let conn = &mut *app.get_conn().await?;
    let check = conn.query(
        "select id from users where id = $1",
        &[&path.user_id]
    ).await?;

    if check.len() == 0 {
        return Err(error::ResponseError::UserIDNotFound(path.user_id));
    }

    let transaction = conn.transaction().await?;
    let _user_access = transaction.execute(
        "delete from user_access where owner = $1 or allowed_for = $1",
        &[&path.user_id]
    ).await?;

    let _mood_entries = transaction.execute(
        "delete from mood_entries where entry in (select id from entries where owner = $1)",
        &[&path.user_id]
    ).await?;

    let _mood_fields = transaction.execute(
        "delete from mood_fields where owner = $1",
        &[&path.user_id]
    ).await?;

    let _text_entries = transaction.execute(
        "delete from text_entries where entry in (select id from entries where owner = $1)",
        &[&path.user_id]
    ).await?;

    let _entries2tags = transaction.execute(
        "delete from entries2tags where entry in (select id from entries where owner = $1)",
        &[&path.user_id]
    ).await?;

    let _entries = transaction.execute(
        "delete from entries where owner = $1",
        &[&path.user_id]
    ).await?;

    let _tags = transaction.execute(
        "delete from tags where owner = $1",
        &[&path.user_id]
    ).await?;

    let _user_sessions = transaction.execute(
        "delete from user_sessions where owner = $1",
        &[&path.user_id]
    ).await?;

    let _users = transaction.execute(
        "delete from users where id = $1",
        &[&path.user_id]
    ).await?;

    transaction.commit().await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::<Option<()>>::build(
            "successful",
            None
        )
    ))
}