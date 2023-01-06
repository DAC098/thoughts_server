use actix_web::HttpRequest;
use actix_web::{web, http, Responder};
use serde::{Deserialize, Serialize};

use crate::db::{users, user_sessions, self};
use crate::security::state::SecurityState;
use crate::security::{self, initiator, InitiatorLookup};
use crate::net::http::error;
use crate::net::http::cookie;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::state;

pub mod verify;

/// GET /auth/session
/// 
/// currently will only serve the html
pub async fn handle_get(
    req: HttpRequest,
    security: security::state::WebSecurityState,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>
) -> error::Result<impl Responder> {
    let conn = &*db.get_conn().await?;

    if response::try_check_if_html_req(&req) {
        let lookup = initiator::from_request(&security, conn, &req).await?;
        
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

/// generates session_id cookie with given duration and value
/// 
/// - domain information is pulled from the security state session object
/// - the path is set to root
/// - max age is set to the duration specified
/// - same site is strict
/// - http only is set to true
fn create_session_cookie<V>(security: &SecurityState, duration: chrono::Duration, value: V) -> cookie::SetCookie
where
    V: Into<String>
{
    let mut session_cookie = cookie::SetCookie::new("session_id", value);
    session_cookie.set_domain(security.get_session().get_domain());
    session_cookie.set_path("/");
    session_cookie.set_max_age(duration);
    session_cookie.set_same_site(cookie::SameSite::Strict);
    session_cookie.set_http_only(true);

    session_cookie
}

/// generates random token and signed token
/// 
/// generates a random 64 byte base64 token and the signs it will the security 
/// state information returning the regular token and the signed token
fn create_token_and_cookie_value(security: &SecurityState) -> error::Result<(String, String)> {
    let bytes = security::get_rand_bytes(36)?;
    let token = base64::encode_config(bytes.as_slice(), base64::URL_SAFE);

    let value = match security::mac::algo_sign_value(
        security.get_signing(),
        security.get_secret().as_bytes(),
        &token,
        "."
    ) {
        Ok(m) => m,
        Err(err) => {
            return Err(error::Error::new().set_source(err))
        }
    };

    Ok((token, value))
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
        let lookup = security::initiator::from_request(&security, &*conn, &req).await?;

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

    let owner: i32 = row.get(0);
    let duration = chrono::Duration::days(7);
    let (token, signed_token) = create_token_and_cookie_value(&security)?;
    let session_cookie = create_session_cookie(&security, duration.clone(), signed_token);

    let issued_on = chrono::Utc::now();
    let expires = issued_on.clone().checked_add_signed(duration).unwrap();
    let mut verified = true;
    let mut verify_option: Option<VerifyOption> = None;

    if let Some(otp) = db::auth_otp::AuthOtp::find_users_id(&transaction, &owner).await? {
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
        let lookup = initiator::from_request(&security, &*conn, &req).await?;

        match lookup {
            InitiatorLookup::Found(initiator::Initiator {session, user: _}) |
            InitiatorLookup::SessionExpired(session) |
            InitiatorLookup::SessionUnverified(session) => {
                let transaction = conn.transaction().await?;

                session.delete(&transaction).await?;

                transaction.commit().await?;
            },
            _ => {}
        }
    }

    let session_cookie = create_session_cookie(&security, chrono::Duration::seconds(0), "");

    JsonBuilder::new(http::StatusCode::OK)
        .insert_header(session_cookie)
        .set_message("session deleted")
        .build(None::<()>)
}