use actix_web::HttpRequest;
use actix_web::{web, http, Responder};
use serde::{Deserialize, Serialize};

use crate::db::{tables::{users, user_sessions}, self};
use crate::security::{self, Initiator, InitiatorLookup, session};
use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::state;
use crate::template;

pub mod verify;

/// GET /auth/session
/// 
/// currently will only serve the html
pub async fn handle_get(
    req: HttpRequest,
    security: security::state::WebSecurityState,
    db: state::WebDbState,
    template: template::WebTemplateState<'_>
) -> error::Result<impl Responder> {
    let conn = db.get_conn().await?;

    if response::try_check_if_html_req(&req) {
        let lookup = InitiatorLookup::from_request(&security, &*conn, &req).await?;

        if lookup.is_valid() {
            Ok(response::redirect_to_path("/entries"))
        } else {
            Ok(response::respond_index_html(&template.into_inner(), None)?)
        }
    } else {
        JsonBuilder::new(http::StatusCode::OK)
            .set_message("no-op")
            .build(None::<()>)
    }
}

#[derive(Serialize)]
#[serde(tag = "method")]
pub enum VerifyOption {
    Totp {
        digits: i16
    }
}

#[derive(Deserialize)]
pub struct LoginBodyJSON {
    username: String,
    password: String
}

/// POST /auth/session
/// 
/// receives the login information from a user request. if accepted then it will
/// send back a successful message with a login session
pub async fn handle_post(
    req: HttpRequest,
    security: security::state::WebSecurityState,
    db: state::WebDbState,
    posted: web::Json<LoginBodyJSON>
) -> error::Result<impl Responder> {
    let mut conn = db.pool.get().await?;

    {
        let lookup = InitiatorLookup::from_request(&security, &*conn, &req).await?;

        match lookup {
            InitiatorLookup::Found(_) => {
                return JsonBuilder::new(http::StatusCode::OK)
                    .set_message("already logged in")
                    .build(None::<()>)
            },
            InitiatorLookup::SessionUnverified(_) => {
                return Err(error::Error::new()
                    .set_status(http::StatusCode::UNAUTHORIZED)
                    .set_name("VerifySession")
                    .set_message("additional verification is required, use previously specified method to verify"))
            }
            _ => {}
        }
    }

    let posted = posted.into_inner();

    let Some(row) = conn.query_opt(
        "select id, hash from users where username = $1 or email = $1",
        &[&posted.username]
    ).await? else {
        return Err(error::Error::new()
            .set_status(http::StatusCode::NOT_FOUND)
            .set_name("UsernameNotFound")
            .set_message("failed to find the requested username"))
    };

    if !security::verify_password(row.get(1), &posted.password)? {
        return Err(error::build::invalid_password());
    }

    let transaction = conn.transaction().await?;
    let token;

    loop {
        let gen = session::create_session_id()?;

        let check = transaction.execute(
            "select * from user_sessions where token = $1",
            &[&gen]
        ).await?;

        if check == 0 {
            token = gen;
            break;
        }
    }

    let owner: i32 = row.get(0);
    let issued_on = chrono::Utc::now();
    let duration = chrono::Duration::days(7);
    let session_cookie = session::create_cookie(
        &security, 
        duration.clone(), 
        session::create_signed(&security, &token)?
    );

    let expires = issued_on.clone()
        .checked_add_signed(duration)
        .ok_or(error::Error::new().set_source("session issued on overflowed"))?;

    let mut verified = true;
    let mut verify_option: Option<VerifyOption> = None;

    if let Some(otp) = db::tables::auth_otp::AuthOtp::find_users_id(&transaction, &owner).await? {
        if otp.verified {
            verified = false;
            verify_option = Some(VerifyOption::Totp {
                digits: otp.digits
            });
        }
    }

    let user_session = user_sessions::UserSession {
        token,
        owner,
        dropped: false,
        expires,
        issued_on,
        verified,
        use_csrf: false
    };

    user_session.insert(&transaction).await?;

    transaction.commit().await?;

    if verify_option.is_some() {
        JsonBuilder::new(http::StatusCode::UNAUTHORIZED)
            .insert_header(session_cookie)
            .set_error("VerifySession")
            .set_message("additional verification is required")
            .build(verify_option)
    } else {
        JsonBuilder::new(http::StatusCode::OK)
            .insert_header(session_cookie)
            .set_message("session created")
            .build(Some(users::find_from_id(&*conn, &row.get(0)).await?))
    }
}

/// DELETE /auth/session
/// 
/// as you can guess it will delete the current session from the database and
/// set a new cookie to have it expire on the client immidiately. will drop
/// the session if it is found, expired, or unverified other wise just drops
/// the cookie from the request
pub async fn handle_delete(
    req: HttpRequest,
    security: security::state::WebSecurityState,
    db: state::WebDbState
) -> error::Result<impl Responder> {
    let mut conn = db.pool.get().await?;

    {
        let lookup = InitiatorLookup::from_request(&security, &*conn, &req).await?;

        match lookup {
            InitiatorLookup::Found(Initiator {session, user: _}) |
            InitiatorLookup::SessionExpired(session) |
            InitiatorLookup::SessionUnverified(session) => {
                let transaction = conn.transaction().await?;

                session.delete(&transaction).await?;

                transaction.commit().await?;
            },
            _ => {}
        }
    }

    let session_cookie = session::create_cookie(&security, chrono::Duration::seconds(0), "");

    JsonBuilder::new(http::StatusCode::OK)
        .insert_header(session_cookie)
        .set_message("session deleted")
        .build(None::<()>)
}