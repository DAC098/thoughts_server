use std::convert::TryFrom;

use actix_web::{web, http, Responder};
use serde::Deserialize;
use rand::RngCore;
use serde::Serialize;

use crate::db::tables::auth_otp;
use crate::net::http::error;
use crate::net::http::response::json::JsonBuilder;
use crate::security::{initiator, otp};
use crate::state;

#[derive(Deserialize)]
pub struct VerifyData {
    value: String
}

#[derive(Serialize)]
pub struct TotpVerified {
    hashes: Vec<String>
}

pub async fn handle_post(
    initiator: initiator::Initiator,
    db: state::WebDbState,
    posted: web::Json<VerifyData>,
) -> error::Result<impl Responder> {
    let mut conn = db.pool.get().await?;
    let posted = posted.into_inner();

    let Some(otp) = auth_otp::AuthOtp::find_users_id(&*conn, &initiator.user.id).await? else {
        return Err(error::Error::new()
            .set_status(http::StatusCode::NOT_FOUND)
            .set_name("TotpNotFound")
            .set_message("user does not have totp enabled"))
    };

    if otp.verified {
        return Err(error::Error::new()
            .set_status(http::StatusCode::BAD_REQUEST)
            .set_name("TotpAlreadyVerified")
            .set_message("totp has already been verified"))
    }

    let Ok(settings) = TryFrom::try_from(&otp) else {
        return Err(error::Error::new());
    };

    match otp::verify_totp_code(&settings, posted.value) {
        otp::VerifyResult::Valid => {},
        otp::VerifyResult::Invalid => {
            return Err(error::Error::new()
                .set_status(http::StatusCode::UNAUTHORIZED)
                .set_name("InvalidTotpCode")
                .set_message("given code is invalid"));
        },
        otp::VerifyResult::InvalidCharacters => {
            return Err(error::Error::new()
                .set_status(http::StatusCode::UNAUTHORIZED)
                .set_name("InvalidTotpCode")
                .set_message("given code containss non numeric characters"));
        },
        otp::VerifyResult::InvalidLength => {
            return Err(error::Error::new()
                .set_status(http::StatusCode::UNAUTHORIZED)
                .set_name("InvalidTotpCode")
                .set_message("given code is not a valid length"))
        },
        _ => {
            return Err(error::Error::new());
        }
    }

    let mut created = 0;
    let mut bytes = [0u8; 5];
    let mut rng = rand::thread_rng();
    let mut hashes: Vec<String> = Vec::with_capacity(10);
    let transaction = conn.transaction().await?;

    transaction.execute(
        "update auth_otp set verified = true where id = $1",
        &[&otp.id]
    ).await?;

    while created < 10 {
        rng.try_fill_bytes(&mut bytes)?;

        let encoded = data_encoding::BASE32.encode(&bytes);
        bytes.fill(0);

        transaction.execute(
            "\
            insert into auth_otp_codes (auth_otp_id, hash) values \
            ($1, $2) \
            on conflict (hash) do nothing",
            &[&otp.id, &encoded]
        ).await?;

        hashes.push(encoded);
        created += 1;
    }

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .set_message("totp has been verified")
        .build(Some(TotpVerified {
            hashes
        }))
}