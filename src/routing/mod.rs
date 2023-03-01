//! http routing handles and methods
//!
//! holds the routing, path, query, etc handles and methods.

use actix_web::Responder;

use crate::net::http::response;

pub mod path;
pub mod query;
pub mod handler;

mod file;
pub use file::*;

mod json;
pub use json::*;

/// placeholder handle
///
/// no additional logic just respondes with 200 okay
#[allow(dead_code)]
pub async fn okay() -> impl Responder {
    response::okay_response()
}
