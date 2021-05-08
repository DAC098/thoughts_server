use actix_web::{web, http, Responder};
use serde::{Deserialize};

use crate::security;
use crate::error;
use crate::state;
use crate::request::from;
use crate::response;

pub struct ChangeUserPasswordJson {
    password: String
}

pub async fn handle_post(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    posted: web::Json<ChangeUserPasswordJson>
) -> error::Result<impl Responder> {
    Ok(response::json::respond_okay())
}