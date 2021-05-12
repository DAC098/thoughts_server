use actix_web::{web, http, HttpRequest, Responder};
use actix_session::{Session};
use tokio_postgres::{Client};
use serde::{Deserialize, Serialize};

use crate::error;
use crate::response;
use crate::state;
use crate::request::from;
use crate::json;

async fn assert_permission_to_read(
    conn: &Client,
    initiator: i32,
    user: i32
) -> error::Result<()> {
    let result = conn.query(
        "select ability, allowed_for from user_access where owner = $1",
        &[&user]
    ).await?;

    if result.len() > 0 {
        for row in result {
            let ability: String = row.get(0);
            let allowed_for: i32 = row.get(1);

            if ability.eq("r") && initiator == allowed_for {
                return Ok(());
            }
        }

        Err(error::ResponseError::PermissionDenied(
            "you do not have permission to read this users information".to_owned()
        ))
    } else {
        Err(error::ResponseError::PermissionDenied(
            "no ability was found for the requested user".to_owned()
        ))
    }
}

#[derive(Serialize)]
pub struct UserJson {
    id: i32,
    full_name: Option<String>,
    username: String,
    ability: String
}

#[derive(Serialize)]
pub struct UserListJson {
    given: Vec<UserJson>,
    allowed: Vec<UserJson>
}

pub async fn handle_get_users(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>,
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
        let allowed_result = conn.query(
            r#"
            select user_access.owner as id,
                   users.full_name as full_name,
                   users.username as username,
                   user_access.ability as ability
            from user_access
            join users on user_access.owner = users.id
            where user_access.allowed_for = $1
            order by user_access.owner
            "#,
            &[&initiator.user.get_id()]
        ).await?;
        let mut allowed = Vec::<UserJson>::with_capacity(allowed_result.len());

        for user in allowed_result {
            allowed.push(UserJson {
                id: user.get(0),
                full_name: user.get(1),
                username: user.get(2),
                ability: user.get(3)
            });
        }

        let given_result = conn.query(
            r#"
            select user_access.allowed_for as id,
                   users.full_name as full_name,
                   users.username as username,
                   user_access.ability as ability
            from user_access
            join users on user_access.allowed_for = users.id
            where user_access.owner = $1
            order by user_access.allowed_for
            "#,
            &[&initiator.user.get_id()]
        ).await?;
        let mut given = Vec::<UserJson>::with_capacity(given_result.len());

        for user in given_result {
            given.push(UserJson {
                id: user.get(0),
                full_name: user.get(1),
                username: user.get(2),
                ability: user.get(3)
            });
        }

        Ok(response::json::respond_json(
            http::StatusCode::OK,
            response::json::MessageDataJSON::build(
                "successful",
                UserListJson {given, allowed}
            )
        ))
    }
}

#[derive(Deserialize)]
pub struct UserPath {
    user_id: i32
}

pub async fn handle_get_users_id(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>,
    path: web::Path<UserPath>,
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
        assert_permission_to_read(conn, initiator.user.get_id(), path.user_id).await?;

        Ok(response::json::respond_json(
            http::StatusCode::OK,
            response::json::MessageDataJSON::build(
                "successful",
                json::search_entries(conn, json::SearchEntriesOptions {
                    owner: path.user_id,
                    from: None,
                    to: None
                }).await?
            )
        ))
    }
}

pub async fn handle_get_users_id_entries(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>,
    path: web::Path<UserPath>,
    info: web::Query<json::QueryEntries>,
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
        assert_permission_to_read(conn, initiator.user.get_id(), path.user_id).await?;

        Ok(response::json::respond_json(
            http::StatusCode::OK,
            response::json::MessageDataJSON::build(
                "successful",
                json::search_entries(conn, json::SearchEntriesOptions {
                    owner: path.user_id,
                    from: info.from,
                    to: info.to
                }).await?
            )
        ))
    }
}

pub async fn handle_get_users_id_mood_fields(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>,
    path: web::Path<UserPath>,
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
        assert_permission_to_read(conn, initiator.user.get_id(), path.user_id).await?;

        Ok(response::json::respond_json(
            http::StatusCode::OK,
            response::json::MessageDataJSON::build(
                "successful",
                json::search_mood_fields(conn, path.user_id).await?
            )
        ))
    }
}

#[derive(Deserialize)]
pub struct UserEntryPath {
    user_id: i32,
    entry_id: i32
}

pub async fn handle_get_users_id_entries_id(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>,
    path: web::Path<UserEntryPath>
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
        assert_permission_to_read(conn, initiator.user.get_id(), path.user_id).await?;

        if let Some(entry) = json::search_entry(conn, path.entry_id).await? {
            if entry.owner != path.user_id {
                Err(error::ResponseError::PermissionDenied(
                    format!("this user does not own the requested entry. user[{}] entry[{}]", path.user_id, path.entry_id)
                ))
            } else {
                Ok(response::json::respond_json(
                    http::StatusCode::OK,
                    response::json::MessageDataJSON::build(
                        "successful",
                        entry
                    )
                ))
            }
        } else {
            Err(error::ResponseError::EntryNotFound(path.entry_id))
        }
    }
}

#[derive(Deserialize)]
pub struct UserFieldPath {
    user_id: i32,
    field_id: i32
}

pub async fn handle_get_users_id_mood_fields_id(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>,
    path: web::Path<UserFieldPath>
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
        assert_permission_to_read(conn, initiator.user.get_id(), path.user_id).await?;

        if let Some(field) = json::search_mood_field(conn, path.field_id).await? {
            if field.owner != path.user_id {
                Err(error::ResponseError::PermissionDenied(
                    format!("this user does not own the requested field. user[{}] field[{}]", path.user_id, path.field_id)
                ))
            } else {
                Ok(response::json::respond_json(
                    http::StatusCode::OK,
                    response::json::MessageDataJSON::<Option<()>>::build(
                        "successful",
                        None
                    )
                ))
            }
        } else {
            Err(error::ResponseError::MoodFieldNotFound(path.field_id))
        }
    }
}