use actix_web::HttpRequest;
use actix_web::{web, http, Responder};
use serde::Deserialize;

use crate::db::user_sessions::UserSession;
use crate::db::{users, user_sessions};

use crate::request::cookie::{SetCookie, SameSite, CookieMap};
use crate::request::initiator_from_request;
use crate::response::json::JsonBuilder;
use crate::response::{self, try_check_if_html_req};
use crate::state;
use crate::security;

use response::error;

/**
 * GET /auth/session
 * currently will only serve the html
 */
pub async fn handle_get(
    req: HttpRequest,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>
) -> error::Result<impl Responder> {
    let conn = &*db.get_conn().await?;

    if try_check_if_html_req(&req) {
        match initiator_from_request(conn, &req).await? {
            Some(_) => Ok(response::redirect_to_path("/entries")),
            None => Ok(response::respond_index_html(&template.into_inner(), None)?)
        }
    } else {
        JsonBuilder::new(http::StatusCode::OK)
            .set_message("no-op")
            .build(None::<()>)
    }
}

#[derive(Deserialize)]
pub struct LoginBodyJSON {
    username: String,
    password: String
}

/**
 * POST /auth/session
 * receives the login information from a user request. if accepted then it will
 * send back a successful message with a login session
 */
pub async fn handle_post(
    security: state::WebSecurityState,
    db: state::WebDbState,
    posted: web::Json<LoginBodyJSON>
) -> error::Result<impl Responder> {
    let mut conn = db.pool.get().await?;
    let posted = posted.into_inner();
    let result = conn.query_opt(
        "select id, hash from users where username = $1 or email = $1",
        &[&posted.username]
    ).await?;

    if result.is_none() {
        return Err(error::ResponseError::UsernameNotFound(posted.username.clone()));
    }
    
    let row = result.unwrap();

    security::verify_password(row.get(1), &posted.password)?;

    let transaction = conn.transaction().await?;
    let bytes = security::get_rand_bytes(64)?;
    let token = base64::encode_config(bytes.as_slice(), base64::URL_SAFE);
    let duration = chrono::Duration::days(7);
    let issued_on = chrono::Utc::now();
    let expires = issued_on.clone().checked_add_signed(duration.clone()).unwrap();

    let user_session = user_sessions::UserSession{
        token,
        owner: row.get(0),
        dropped: false,
        expires,
        issued_on,
        use_csrf: false
    };

    user_session.insert(&transaction).await?;
    
    let mut session_cookie = SetCookie::new("session_id", user_session.token.to_string());
    session_cookie.set_domain(security.get_session().get_domain());
    session_cookie.set_path("/");
    session_cookie.set_max_age(duration);
    session_cookie.set_same_site(SameSite::Strict);
    session_cookie.set_http_only(true);

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .insert_header(session_cookie)
        .set_message("session created")
        .build(Some(users::find_from_id(&*conn, &row.get(0)).await?))
}

pub async fn handle_delete(
    req: HttpRequest,
    security: state::WebSecurityState,
    db: state::WebDbState
) -> error::Result<impl Responder> {
    let mut conn = db.pool.get().await?;
    let cookies = CookieMap::from(&req);

    if let Some(token) = cookies.get_value_ref("session_id") {
        let transaction = conn.transaction().await?;
        UserSession::delete_via_token(&transaction, &token).await?;

        transaction.commit().await?;
    }

    let mut session_cookie = SetCookie::new("session_id", "");
    session_cookie.set_domain(security.get_session().get_domain());
    session_cookie.set_path("/");
    session_cookie.set_max_age(chrono::Duration::seconds(0));
    session_cookie.set_same_site(SameSite::Strict);
    session_cookie.set_http_only(true);

    JsonBuilder::new(http::StatusCode::OK)
        .insert_header(session_cookie)
        .set_message("session deleted")
        .build_empty()
}