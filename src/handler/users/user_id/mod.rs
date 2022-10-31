use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;
use lettre::message::Mailbox;

use crate::db;

use crate::security::Initiator;
use crate::security::initiator_from_request;
use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::state;
use crate::security;
use crate::email;
use crate::util;
use crate::routing::path;

pub async fn handle_get(
    req: HttpRequest,
    security: state::WebSecurityState,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
    path: web::Path<path::params::UserPath>,
) -> error::Result<impl Responder> {
    let accept_html = response::try_check_if_html_req(&req);
    let conn = &*db.get_conn().await?;
    let initiator_opt = initiator_from_request(&security, conn, &req).await?;

    if accept_html {
        return if initiator_opt.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(initiator_opt.unwrap().user))?)
        } else {
            let redirect = format!("/auth/login?jump_to=/users/{}", path.user_id);
            Ok(response::redirect_to_path(redirect.as_str()))
        }
    } else if initiator_opt.is_none() {
        return Err(error::ResponseError::Session)
    }
    
    let initiator = initiator_opt.unwrap();

    if !security::permissions::has_permission(
        conn, 
        &initiator.user.id, 
        db::permissions::rolls::USERS, 
        &[
            db::permissions::abilities::READ,
            db::permissions::abilities::READ_WRITE
        ],
        Some(&path.user_id)
    ).await? {
        return Err(error::ResponseError::PermissionDenied(
            "you do not have permission to read this users information".into()
        ))
    }

    if let Some(user) = db::users::find_from_id(conn, &path.user_id).await? {
        JsonBuilder::new(http::StatusCode::OK)
            .build(Some(user))
    } else {
        Err(error::ResponseError::UserIDNotFound(path.user_id))
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
    email: String
}

#[derive(Deserialize)]
pub struct PutJson {
    user: PutUser,
    data: PutUserData
}

pub async fn handle_put(
    initiator: Initiator,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
    email: state::WebEmailState,
    server_info: state::WebServerInfoState,
    posted: web::Json<PutJson>,
    path: web::Path<path::params::UserPath>,
) -> error::Result<impl Responder> {
    let conn = &mut *db.get_conn().await?;
    let posted = posted.into_inner();

    if !security::permissions::has_permission(
        conn, 
        &initiator.user.id, 
        db::permissions::rolls::USERS, 
        &[
            db::permissions::abilities::READ_WRITE
        ],
        Some(&path.user_id)
    ).await? {
        return Err(error::ResponseError::PermissionDenied(
            "you do not have permission to edit another user".into()
        ));
    }

    let original = db::users::find_from_id(conn, &path.user_id).await?;

    if original.is_none() {
        return Err(error::ResponseError::UserIDNotFound(path.user_id));
    }

    let original = original.unwrap();
    let mut email_verified: bool = false;
    let mut email_value: Option<String> = None;
    let mut to_mailbox: Option<Mailbox> = None;

    if email.is_enabled() {
        let check = email::validate_new_email(&*conn, &posted.user.email, &path.user_id).await?;

        if let Some(current_email) = original.email {
            if current_email == posted.user.email {
                email_verified = original.email_verified;
            }
        }

        to_mailbox = Some(Mailbox::new(None, check));
        email_value = Some(posted.user.email);
    }

    let transaction = conn.transaction().await?;

    let result = transaction.query_one(
        "\
        update users \
        set username = $2, \
            level = $3, \
            email = $4, \
            email_verified = $5 \
        where id = $1 \
        returning id, username, level, email",
        &[
            &path.user_id,
            &posted.user.username,
            &posted.user.level,
            &email_value,
            &email_verified
        ]
    ).await?;

    if email.is_enabled() && !email_verified {
        email::send_verify_email(
            &transaction, 
            &server_info, 
            &email, 
            &template, 
            &path.user_id,
            to_mailbox.unwrap()
        ).await?;
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

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(db::composed::ComposedFullUser {
            user: db::users::User {
                id: path.user_id,
                username: result.get(1),
                level: result.get(2),
                email: result.get(3),
                email_verified
            },
            data: user_data,
            access: Vec::new()
        }))
}

pub async fn handle_delete(
    initiator: Initiator,
    db: state::WebDbState,
    path: web::Path<path::params::UserPath>,
) -> error::Result<impl Responder> {
    let conn = &mut *db.get_conn().await?;

    if !security::permissions::has_permission(
        conn, 
        &initiator.user.id, 
        db::permissions::rolls::USERS, 
        &[
            db::permissions::abilities::READ_WRITE
        ],
        Some(&path.user_id)
    ).await? {
        return Err(error::ResponseError::PermissionDenied(
            format!("you do not have permission to delete another user")
        ));
    }

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

    JsonBuilder::new(http::StatusCode::OK)
        .build(None::<()>)
}