use std::fmt::Write;
use std::collections::HashMap;

use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;
use lettre::{Message, Transport};
use lettre::message::Mailbox;

use tlib::db;

pub mod user_id;

use crate::request::{initiator_from_request, Initiator};
use crate::response;
use crate::state;
use crate::security;
use crate::util;
use crate::email;

use response::error;

#[derive(Deserialize)]
pub struct UserSearchQuery {
    level: Option<i32>,
    full_name: Option<String>,
    username: Option<String>
}

pub async fn handle_get(
    req: HttpRequest,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
    info: web::Query<UserSearchQuery>,
) -> error::Result<impl Responder> {
    let accept_html = response::try_check_if_html_req(&req);
    let conn = &*db.get_conn().await?;
    let initiator_opt = initiator_from_request(conn, &req).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(initiator_opt.unwrap().user))?)
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
pub struct PostUserData {
    prefix: Option<String>,
    suffix: Option<String>,
    first_name: String,
    last_name: String,
    middle_name: Option<String>,
    dob: String
}

#[derive(Deserialize)]
pub struct PostUser {
    username: String,
    password: String,
    email: String,
    full_name: Option<String>,
    level: i32
}

#[derive(Deserialize)]
pub struct PostUserJson {
    user: PostUser,
    data: PostUserData,
    access: Vec<i32>
}

pub async fn handle_post(
    initiator: Initiator,
    db: state::WebDbState,
    email: state::WebEmailState,
    server_info: state::WebServerInfoState,
    posted: web::Json<PostUserJson>,
) -> error::Result<impl Responder> {
    let email = email.into_inner();
    let server_info = server_info.into_inner();
    let posted = posted.into_inner();

    security::assert::is_admin(&initiator)?;

    let conn = &mut *db.get_conn().await?;
    let (found_username, found_email) = db::users::check_username_email(
        conn, &posted.user.username, &posted.user.email
    ).await?;

    if found_username {
        return Err(error::ResponseError::UsernameExists(posted.user.username.clone()));
    }

    if found_email {
        return Err(error::ResponseError::EmailExists(posted.user.email.clone()))
    }

    let email_verified: bool = false;
    let mut email_value: Option<String> = None;
    let mut to_mailbox: Option<Mailbox> = None;

    if email.is_enabled() {
        let to_mailbox_result = posted.user.email.parse::<Mailbox>();

        if to_mailbox_result.is_err() {
            return Err(error::ResponseError::Validation(
                format!("given email address is invalid. {}", posted.user.email)
            ));
        } else {
            to_mailbox = Some(to_mailbox_result.unwrap());
        }

        email_value = Some(posted.user.email);
    }

    let hash = security::generate_new_hash(&posted.user.password)?;
    let transaction = conn.transaction().await?;

    let user_result = transaction.query_one(
        "\
        insert into users (level, username, full_name, hash, email) \
        values ($1, $2, $3, $4, $5) \
        returning id",
        &[
            &posted.user.level, 
            &posted.user.username, 
            &posted.user.full_name, 
            &hash, 
            &email_value,
            &email_verified
        ]
    ).await?;

    let user_id = user_result.get(0);

    if email.is_enabled() && email.can_get_transport() && email.has_from() {
        let rand_bytes = security::get_rand_bytes(32)?;
        let hex_str = util::hex_string(rand_bytes)?;
        let issued = util::time::now();

        transaction.execute(
            "\
            insert into email_verification (owner, key_id, issued)\
            values ($1, $2, $3)",
            &[&user_id, &hex_str, &issued]
        ).await?;

        let email_message = Message::builder()
        .from(email.get_from().unwrap())
        .to(to_mailbox.unwrap())
        .subject("Verify Changed Email")
        .multipart(email::message_body::verify_email_body(
            server_info.url_origin(), hex_str
        ))?;

        email.get_transport()?.send(&email_message)?;
    }

    let user_data = {
        let prefix = util::string::trimmed_optional_string(posted.data.prefix);
        let suffix = util::string::trimmed_optional_string(posted.data.suffix);
        let first_name = util::string::trimmed_string(posted.data.first_name);
        let last_name = util::string::trimmed_string(posted.data.last_name);
        let middle_name = util::string::trimmed_optional_string(posted.data.middle_name);
        let dob: chrono::NaiveDate;

        if let Ok(date) = posted.data.dob.parse() {
            dob = date;
        } else {
            let mut message = String::from("invalid date format given. format: YYYY-MM-DD given: \"");
            message.reserve(posted.data.dob.len() + 1);
            message.push_str(&posted.data.dob);
            message.push('"');

            return Err(error::ResponseError::Validation(message))
        }

        transaction.execute(
            "\
            insert into user_data (owner, prefix, suffix, first_name, last_name, middle_name, dob) \
            values ($1, $2, $3, $4, $5, $6, $7)",
            &[
                &user_id,
                &prefix,
                &suffix,
                &first_name,
                &last_name,
                &middle_name,
                &dob
            ]
        ).await?;

        db::user_data::UserData {
            owner: user_id,
            prefix, suffix,
            first_name, last_name, middle_name,
            dob
        }
    };

    let mut user_access: Vec<db::composed::ComposedUserAccess> = Vec::new();

    {
        let user_level = posted.user.level;
        let is_manager = user_level == (db::users::Level::Manager as i32);
        let check_level: i32 = if is_manager { db::users::Level::User as i32 } else { db::users::Level::Manager as i32 };
        let mut invalid: Vec<String> = Vec::with_capacity(posted.access.len());
        let mut user_mapping: HashMap<i32, db::users::User> = HashMap::new();

        let check_result = transaction.query(
            "\
            select users.id, \
                   users.username, \
                   users.level, \
                   users.full_name, \
                   users.email, \
                   users.email_verified \
            from users \
            where users.id = any($1)",
            &[&posted.access]
        ).await?;

        for check in check_result {
            let user = db::users::User {
                id: check.get(0),
                username: check.get(1),
                level: check.get(2),
                full_name: check.get(3),
                email: check.get(4),
                email_verified: check.get(5)
            };

            if user.level != check_level {
                invalid.push(user.username);
            } else if !user_mapping.contains_key(&user.id) {
                user_mapping.insert(user.id, user);
            }
        }

        if !invalid.is_empty() {
            return Err(error::ResponseError::Validation(
                format!("some of the users requested are not the appropriate level, usernames: {:?}", invalid.join(", "))
            ));
        }

        user_access.reserve(user_mapping.len());

        let ability = "r";
        let first_arg = if is_manager { "owner" } else { "allowed_for" };
        let second_arg = if is_manager { "allowed_for" } else { "owner" };
        let mut insert_query_list: Vec<String> = Vec::with_capacity(user_mapping.len());
        let mut insert_query_slice = db::query::QueryParams::with_capacity(2 + user_mapping.len());
        insert_query_slice.push(&user_id);
        insert_query_slice.push(&ability);

        for (id, _user) in &user_mapping {
            insert_query_list.push(format!("($1, $2, ${})", insert_query_slice.push(id)));
        }

        let insert_query_str = format!(
            "insert into user_access ({}, ability, {}) values {} returning {}",
            first_arg, second_arg, insert_query_list.join(", "), second_arg
        );

        let inserted_records = transaction.query(
            insert_query_str.as_str(),
            insert_query_slice.slice()
        ).await?;

        for record in inserted_records {
            let id: i32 = record.get(0);
            let user_info = user_mapping.remove(&id).unwrap();

            user_access.push(db::composed::ComposedUserAccess {
                user: user_info,
                access: db::user_access::UserAccess {
                    owner: if is_manager { user_id } else { id },
                    ability: "r".to_owned(),
                    allowed_for: if is_manager { id } else { user_id }
                }
            });
        }
    }

    transaction.commit().await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "created account",
            db::composed::ComposedFullUser {
                user: db::users::User {
                    id: user_result.get(0),
                    username: posted.user.username.clone(),
                    full_name: posted.user.full_name.clone(),
                    email: email_value,
                    email_verified: false,
                    level: posted.user.level
                },
                data: user_data,
                access: user_access
            }
        )
    ))
}