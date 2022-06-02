use std::collections::HashMap;

use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;
use lettre::{Message, Transport};
use lettre::message::Mailbox;

use tlib::db;

use crate::request::{initiator_from_request, Initiator};
use crate::response;
use crate::state;
use crate::security::assert;
use crate::util;
use crate::email;

use response::error;

#[derive(Deserialize)]
pub struct UserIdPath {
    user_id: i32
}

pub async fn handle_get(
    req: HttpRequest,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
    path: web::Path<UserIdPath>
) -> error::Result<impl Responder> {
    let accept_html = response::try_check_if_html_req(&req);
    let conn = &*db.get_conn().await?;
    let initiator_opt = initiator_from_request(conn, &req).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(initiator_opt.unwrap().user))?)
        } else {
            Ok(response::redirect_to_login(&req))
        }
    } else if initiator_opt.is_none() {
        Err(error::ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();

        assert::is_admin(&initiator)?;

        if let Some(user_record) = db::composed::ComposedUser::find_from_id(conn, &path.user_id).await? {
            let access = db::composed::ComposedUserAccess::find(
                conn, &user_record.user.id,
                user_record.user.level == (db::users::Level::Manager as i32)
            ).await?;

            Ok(response::json::respond_json(
                http::StatusCode::OK,
                response::json::MessageDataJSON::build(
                    "successful",
                    db::composed::ComposedFullUser {
                        user: user_record.user,
                        data: user_record.data,
                        access
                    }
                )
            ))
        } else {
            Err(error::ResponseError::UserIDNotFound(path.user_id))
        }
    }
}

#[derive(Deserialize)]
pub struct PutUserData {
    prefix: Option<String>,
    suffix: Option<String>,
    first_name: String,
    last_name: String,
    middle_name: Option<String>,
    dob: String
}

#[derive(Deserialize)]
pub struct PutUser {
    username: String,
    level: i32,
    full_name: Option<String>,
    email: String
}

#[derive(Deserialize)]
pub struct PutJson {
    user: PutUser,
    data: PutUserData,
    access: Vec<i32>
}

pub async fn handle_put(
    initiator: Initiator,
    db: state::WebDbState,
    email: state::WebEmailState,
    server_info: state::WebServerInfoState,
    posted: web::Json<PutJson>,
    path: web::Path<UserIdPath>,
) -> error::Result<impl Responder> {
    let conn = &mut *db.get_conn().await?;
    let posted = posted.into_inner();

    assert::is_admin(&initiator)?;

    let original = db::users::find_from_id(conn, &path.user_id).await?;

    if original.is_none() {
        return Err(error::ResponseError::UserIDNotFound(path.user_id));
    }

    let original = original.unwrap();
    let mut email_verified: bool = false;
    let mut email_value: Option<String> = None;
    let mut to_mailbox: Option<Mailbox> = None;

    if email.is_enabled() {
        let to_mailbox_result = posted.user.email.parse::<Mailbox>();

        if to_mailbox_result.is_err() {
            let mut message = String::from("given email address is invalid. given: \"");
            message.reserve(posted.user.email.len() + 1);
            message.push_str(&posted.user.email);
            message.push('"');

            return Err(error::ResponseError::Validation(message));
        } else {
            to_mailbox = Some(to_mailbox_result.unwrap());
        }

        if let Some(check) = conn.query_opt("select id from users where email = $1", &[&posted.user.email]).await? {
            if check.get::<usize, i32>(0) != original.id {
                return Err(error::ResponseError::EmailExists(posted.user.email))
            }
        }

        if let Some(current_email) = original.email {
            email_verified = if current_email == posted.user.email {
                original.email_verified
            } else {
                false
            };
        }

        email_value = Some(posted.user.email);
    }

    let transaction = conn.transaction().await?;

    let result = transaction.query_one(
        "\
        update users \
        set username = $2, \
            level = $3, \
            full_name = $4, \
            email = $5, \
            email_verified = $6 \
        where id = $1 \
        returning id, username, level, full_name, email",
        &[
            &path.user_id,
            &posted.user.username,
            &posted.user.level,
            &posted.user.full_name,
            &email_value,
            &email_verified
        ]
    ).await?;

    if email.is_enabled() && !email_verified && email.can_get_transport() && email.has_from() {
        let mut rand_bytes: [u8; 32] = [0; 32];
        openssl::rand::rand_bytes(&mut rand_bytes)?;
        let hex_str = util::hex_string(&rand_bytes)?;
        let issued = util::time::now();

        transaction.execute(
            "\
            insert into email_verifications (owner, key_id, issued) \
            values ($1, $2, $3) \
            on conflict on constraint email_verifications_pkey do update \
            set key_id = excluded.key_id, \
                issued = excluded.issued",
            &[&original.id, &hex_str, &issued]
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
            update user_data \
            set prefix = $2, \
                suffix = $3, \
                first_name = $4, \
                last_name = $5, \
                middle_name = $6, \
                dob = $7 \
            where owner = $1",
            &[
                &path.user_id,
                &prefix, &suffix,
                &first_name, &last_name, &middle_name,
                &dob
            ]
        ).await?;

        db::user_data::UserData {
            owner: path.user_id,
            prefix, suffix,
            first_name, last_name, middle_name,
            dob
        }
    };

    let mut user_access: Vec<db::composed::ComposedUserAccess> = vec!();
    
    {
        let user_level: i32 = result.get(2);
        let is_manager = user_level == (db::users::Level::Manager as i32);
        let check_level: i32 = if is_manager { 20 } else { 10 };
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

        if invalid.len() > 0 {
            return Err(error::ResponseError::Validation(
                format!("some of the users requested are not the appropriate level, usernames: {:?}", invalid.join(", "))
            ));
        }

        user_access.reserve(user_mapping.len());

        if is_manager {
            transaction.execute(
                "delete from user_access where owner = $1",
                &[&path.user_id]
            ).await?;
        } else {
            transaction.execute(
                "delete from user_access where allowed_for = $1",
                &[&path.user_id]
            ).await?;
        }

        let ability = "r";
        // the static field for the current user
        let first_arg = if is_manager { "owner" } else { "allowed_for" };
        // the dynamic field that will assigned for the user_access list given
        let second_arg = if is_manager { "allowed_for" } else { "owner" };
        let mut insert_query_list: Vec<String> = vec!();
        let mut insert_query_slice: db::query::QueryParams = db::query::QueryParams::with_capacity(2);
        insert_query_slice.push(&path.user_id);
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
                    owner: if is_manager {
                        path.user_id
                    } else {
                        id
                    },
                    ability: "r".to_owned(),
                    allowed_for: if is_manager {
                        id
                    } else {
                        path.user_id
                    }
                }
            });
        }
    }

    transaction.commit().await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful",
            db::composed::ComposedFullUser {
                user: db::users::User {
                    id: path.user_id,
                    username: result.get(1),
                    level: result.get(2),
                    full_name: result.get(3),
                    email: result.get(4),
                    email_verified
                },
                data: user_data,
                access: user_access
            }
        )
    ))
}

pub async fn handle_delete(
    initiator: Initiator,
    db: state::WebDbState,
    path: web::Path<UserIdPath>,
) -> error::Result<impl Responder> {
    if initiator.user.level != 1 {
        return Err(error::ResponseError::PermissionDenied(
            format!("you do not have permission to delete another user")
        ));
    }

    let conn = &mut *db.get_conn().await?;
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

    let _entry_markers = transaction.execute(
        "delete from entry_markers where entry in (select id from entries where owner = $1)",
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
        response::json::only_message("successful")
    ))
}