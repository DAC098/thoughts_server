//! handles creating totp 2FA for accounts

use actix_web::{web, http, Responder};
use serde::{Deserialize, Serialize};

use crate::net::http::error;
use crate::net::http::response::json::JsonBuilder;
use crate::security::{self, initiator};
use crate::state;
use crate::db::tables::auth_otp;

pub mod verify;

#[derive(Deserialize)]
pub struct TotpOptions {
    algo: Option<String>,
    digits: Option<i16>,
    step: Option<i16>,
}

#[derive(Serialize)]
pub struct TotpData {
    algo: String,
    digits: i16,
    step: i16,
    secret: String
}

/// handles creating the desired totp 2FA process
///
/// POST /auth/totp
///
/// allows SHA1, SHA256, SHA512 hashing algorithms; 1 - 10 digits; and step 
/// greater than 0. once it has been created it will need to be verified to
/// take effect
pub async fn handle_post(
    initiator: initiator::Initiator,
    db: state::WebDbState,
    posted: web::Json<TotpOptions>
) -> error::Result<impl Responder> {
    let mut conn = db.pool.get().await?;
    let posted = posted.into_inner();

    let algo = {
        if let Some(given) = posted.algo {
            let Ok(algo) = auth_otp::Algo::try_from_str(given) else {
                return Err(error::build::validation("invalid algo value. can be SHA1, SHA256, or SHA512"));
            };

            algo
        } else {
            auth_otp::Algo::SHA1
        }
    };

    let digits = {
        if let Some(given) = posted.digits {
            if given > 0 && given <= 10 {
                given
            } else {
                return Err(error::build::validation("invalid digits value. value must be greater than 0 and less than 11"));
            }
        } else {
            6
        }
    };

    let step = {
        if let Some(given) = posted.step {
            if given > 0 {
                given
            } else {
                return Err(error::build::validation("invalid step value. value must be greater than 0"));
            }
        } else {
            30
        }
    };

    let secret = security::get_rand_bytes(25)?;
    let verified = false;
    let algo_int = algo.clone().into_i16();

    let transaction = conn.transaction().await?;

    transaction.execute(
        "\
        insert into auth_otp (users_id, algo, secret, digits, step, verified) values \
        ($1, $2, $3, $4, $5, $6)",
        &[&initiator.user.id, &algo_int, &secret, &digits, &step, &verified]
    ).await?;

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .set_message("verify new totp 2fa")
        .build(Some(TotpData {
            algo: algo.into(),
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

    if let Some(row) = conn.query_opt(
        "select id from auth_otp where users_id = $1",
        &[&initiator.user.id]
    ).await? {
        let auth_otp_id: i32 = row.get(0);
        let transaction = conn.transaction().await?;

        transaction.execute(
            "delete from auth_otp_codes where auth_otp_id = $1", 
            &[&auth_otp_id]
        ).await?;

        transaction.execute(
            "delete from auth_otp where id = $1",
            &[&auth_otp_id]
        ).await?;

        transaction.commit().await?;
    }

    JsonBuilder::new(http::StatusCode::OK)
        .set_message("deleted totp requirements")
        .build(None::<()>)
}
