use actix_web::{web, http, Responder};
use serde::{Deserialize, Serialize};

use crate::net::http::error;
use crate::net::http::response::json::JsonBuilder;
use crate::security::{self, initiator};
use crate::state;
use crate::db::auth_otp;

pub mod verify;

#[derive(Deserialize)]
pub struct TotpOptions {
    algo: String,
    digits: i16,
    step: i16,
}

#[derive(Serialize)]
pub struct TotpData {
    algo: String,
    digits: i16,
    step: i16,
    secret: String
}

pub async fn handle_post(
    initiator: initiator::Initiator,
    db: state::WebDbState,
    posted: web::Json<TotpOptions>
) -> error::Result<impl Responder> {
    let mut conn = db.pool.get().await?;
    let posted = posted.into_inner();

    let Ok(algo) = auth_otp::Algo::try_from_str(posted.algo) else {
        return Err(error::build::validation("invalid algo value. can be SHA1, SHA256, or SHA512"));
    };
    let digits = if posted.digits > 0 {
        posted.digits
    } else {
        return Err(error::build::validation("invalid digits value. value must be greater than 0"));
    };
    let step = if posted.step > 0 {
        posted.step
    } else {
        return Err(error::build::validation("invalid step value. value must be greater than 0"));
    };
    let secret = security::get_rand_bytes(32)?;
    let verified = false;
    let algo_int = algo.clone().into_i16();

    let transaction = conn.transaction().await?;

    transaction.execute(
        "\
        insert into auth_otp (users_id, algo, secret, digits, step, verified) values \
        ($1, $2, $3, $4, $5, $6, $7)",
        &[&initiator.user.id, &algo_int, &secret, &digits, &step, &verified]
    ).await?;

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .set_message("verify new totp 2fa")
        .build(Some(TotpData {
            algo: algo.into_string(),
            digits,
            step,
            secret: data_encoding::BASE32.encode(secret.as_slice())
        }))
}

pub async fn handle_delete(
    initiator: initiator::Initiator,
    db: state::WebDbState
) -> error::Result<impl Responder> {
    let mut conn = db.pool.get().await?;
    let transaction = conn.transaction().await?;

    transaction.execute(
        "delete from auth_otp where users_id = $1",
        &[&initiator.user.id]
    ).await?;

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .set_message("deleted totp requirements")
        .build(None::<()>)
}