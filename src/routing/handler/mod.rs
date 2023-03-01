//! contains all http request handlers
//!
//! each file or folder is a path segment that can handle an http request. in
//! each file contains a handle_{method} to indicate what http method it is
//! designed to handle.


use actix_web::{HttpRequest, Responder};

use crate::security::{self, InitiatorLookup};
use crate::net::http::{error, response};
use crate::state;

pub mod ping;
pub mod auth;
pub mod entries;
pub mod custom_fields;
pub mod users;
pub mod account;
pub mod backup;
pub mod tags;
pub mod email;
pub mod global;
pub mod groups;

pub async fn handle_get(
    req: HttpRequest,
    security: security::state::WebSecurityState,
    db: state::WebDbState,
) -> error::Result<impl Responder> {
    let conn = &*db.get_conn().await?;
    let lookup = InitiatorLookup::from_request(&security, conn, &req).await?;

    if lookup.is_valid() {
        Ok(response::redirect_to_path("/entries"))
    } else {
        Ok(response::redirect_to_path("/auth/session"))
    }
}

