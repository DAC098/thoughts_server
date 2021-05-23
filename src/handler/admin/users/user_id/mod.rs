use std::collections::{HashMap};

use actix_web::{web, http, HttpRequest, Responder};
use actix_session::{Session};
use serde::{Serialize, Deserialize};

use crate::error;
use crate::request::from;
use crate::response;
use crate::state;
use crate::db;

#[derive(Deserialize)]
pub struct UserIdPath {
    user_id: i32
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserAccessInfoJson {
    id: i32,
    username: String,
    full_name: Option<String>,
    ability: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserInfoJson {
    id: i32,
    username: String,
    level: i32,
    full_name: Option<String>,
    email: Option<String>,
    user_access: Vec<UserAccessInfoJson>
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
            let redirect = format!("/auth/login?jump_to=/admin/users/{}", path.user_id);
            Ok(response::redirect_to_path(redirect.as_str()))
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
                let user_id: i32 = result[0].get(0);
                let user_level: i32 = result[0].get(2);
                let query = format!(
                    r#"
                    select users.id,
                           users.username,
                           users.full_name,
                           user_access.ability
                    from users
                    join user_access on users.id = user_access.{}
                    where user_access.{} = $1
                    "#, 
                    if user_level == 20 {"allowed_for"} else {"owner"},
                    if user_level == 20 {"owner"} else {"allowed_for"}
                );
                let list_result = conn.query(query.as_str(), &[&user_id]).await?;
                let mut user_access: Vec<UserAccessInfoJson> = Vec::with_capacity(list_result.len());

                for row in list_result {
                    user_access.push(UserAccessInfoJson {
                        id: row.get(0),
                        username: row.get(1),
                        full_name: row.get(2),
                        ability: row.get(3)
                    });
                }

                Ok(response::json::respond_json(
                    http::StatusCode::OK,
                    response::json::MessageDataJSON::build(
                        "successful",
                        UserInfoJson {
                            id: result[0].get(0),
                            username: result[0].get(1),
                            level: result[0].get(2),
                            full_name: result[0].get(3),
                            email: result[0].get(4),
                            user_access
                        }
                    )
                ))
            }
        }
    }
}

#[derive(Deserialize)]
pub struct PutUserAccess {
    id: i32
}

#[derive(Deserialize)]
pub struct PutUserJson {
    username: String,
    level: i32,
    full_name: Option<String>,
    email: Option<String>,
    user_access: Vec<PutUserAccess>
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
    let transaction = conn.transaction().await?;

    let result = transaction.query_one(
        r#"
        update users
        set username = $2,
            level = $3,
            full_name = $4,
            email = $5
        where id = $1
        returning id, username, level, full_name, email
        "#, 
        &[&path.user_id, &posted.username, &posted.level, &posted.full_name, &posted.email]
    ).await?;

    let mut user_access: Vec<UserAccessInfoJson> = vec!();
    
    {
        let user_level: i32 = result.get(2);
        let check_level: i32 = if user_level == 10 { 20 } else { 10 };
        let mut id_list: Vec<i32> = Vec::with_capacity(posted.user_access.len());
        let mut invalid: Vec<String> = Vec::with_capacity(posted.user_access.len());
        let mut user_mapping: HashMap<i32, db::users::User> = HashMap::new();

        for user in &posted.user_access {
            id_list.push(user.id);
        }

        let check_result = transaction.query(
            "select users.id, users.username, users.level, users.full_name, users.email from users where users.id = any($1)",
            &[&id_list]
        ).await?;

        for check in check_result {
            let user = db::users::User {
                id: check.get(0),
                username: check.get(1),
                level: check.get(2),
                full_name: check.get(3),
                email: check.get(4)
            };

            if user.level != check_level {
                invalid.push(user.username);
            } else if !user_mapping.contains_key(&user.id) {
                user_mapping.insert(user.id, user);
            }
        }

        if invalid.len() > 0 {
            return Err(error::ResponseError::Validation(
                format!("some of the users requested are not the appropriate level, usernames: {:?}", invalid.join(", "))
            ));
        }

        user_access.reserve(user_mapping.len());

        transaction.execute(
            "delete from user_access where owner = $1 or allowed_for = $1",
            &[&path.user_id]
        ).await?;

        let ability = "r";
        // the static field for the current user
        let first_arg = if user_level == 10 { "allowed_for" } else { "owner" };
        // the dynamic field that will assigned for the user_access list given
        let second_arg = if user_level == 10 { "owner" } else { "allowed_for" };
        let mut insert_arg_count: usize = 3;
        let mut insert_query_list: Vec<String> = vec!();
        let mut insert_query_slice: Vec<&(dyn tokio_postgres::types::ToSql + std::marker::Sync)> = vec![&path.user_id, &ability];

        for (id, _user) in &user_mapping {
            insert_query_list.push(format!("($1, $2, ${})", insert_arg_count));
            insert_query_slice.push(id);
            insert_arg_count += 1;
        }

        let insert_query_str = format!(
            "insert into user_access ({}, ability, {}) values {} returning {}",
            first_arg, second_arg, insert_query_list.join(", "), second_arg
        );

        let inserted_records = transaction.query(
            insert_query_str.as_str(),
            &insert_query_slice[..]
        ).await?;

        for record in inserted_records {
            let id: i32 = record.get(0);
            let user_info = user_mapping.remove(&id).unwrap();

            user_access.push(UserAccessInfoJson {
                id: user_info.id,
                username: user_info.username,
                full_name: user_info.full_name,
                ability: "r".to_owned()
            });
        }
    }

    transaction.commit().await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful",
            UserInfoJson {
                id: path.user_id,
                username: result.get(1),
                level: result.get(2),
                full_name: result.get(3),
                email: result.get(4),
                user_access
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

    let _custom_field_entries = transaction.execute(
        "delete from custom_field_entries where entry in (select id from entries where owner = $1)",
        &[&path.user_id]
    ).await?;

    let _custom_fields = transaction.execute(
        "delete from custom_fields where owner = $1",
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