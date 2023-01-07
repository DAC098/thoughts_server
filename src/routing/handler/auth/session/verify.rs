use std::convert::TryFrom;

use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;

use crate::net::http::error;
use crate::net::http::response::json::JsonBuilder;
use crate::security::{self, otp, initiator::InitiatorLookup};
use crate::state;
use crate::db::{self, tables::users};

#[derive(Deserialize)]
#[serde(tag = "method")]
pub enum VerifyMethod {
    Totp {
        value: String
    },
    TotpHash {
        value: String
    }
}

pub async fn handle_post(
    req: HttpRequest,
    security: security::state::WebSecurityState,
    db: state::WebDbState,
    posted: web::Json<VerifyMethod>
) -> error::Result<impl Responder> {
    let mut conn = db.pool.get().await?;
    let session;

    {
        let lookup = InitiatorLookup::from_request(&security, &*conn, &req).await?;

        match lookup {
            InitiatorLookup::Found(_) => {
                return JsonBuilder::new(http::StatusCode::OK)
                    .set_message("session already verified")
                    .build(None::<()>)
            },
            InitiatorLookup::SessionUnverified(to_use) => {
                // its what they are here for
                session = to_use;
            },
            _ => {
                return Err(lookup.get_error().unwrap());
            }
        }
    }

    let transaction = conn.transaction().await?;

    match posted.into_inner() {
        VerifyMethod::Totp { value } => {
            let Some(otp) = db::tables::auth_otp::AuthOtp::find_users_id(&transaction, &session.owner).await? else {
                return Err(error::Error::new()
                    .set_status(http::StatusCode::NOT_FOUND)
                    .set_name("TotpNotFound")
                    .set_message("user does not have totp enabled"));
            };

            if !otp.verified {
                return Err(error::Error::new()
                    .set_status(http::StatusCode::UNAUTHORIZED)
                    .set_name("TotpUnverified")
                    .set_message("user totp has not been verified"))
            }

            let Ok(settings) = TryFrom::try_from(&otp) else {
                return Err(error::Error::new());
            };

            match security::otp::verify_totp_code(&settings, value) {
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
            };
        },
        VerifyMethod::TotpHash { value } => {
            let Some(otp) = db::tables::auth_otp::AuthOtp::find_users_id(&transaction, &session.owner).await? else {
                return Err(error::Error::new()
                    .set_status(http::StatusCode::NOT_FOUND)
                    .set_name("TotpNotFound")
                    .set_message("user does not have totp enabled"))
            };

            if !otp.verified {
                return Err(error::Error::new()
                    .set_status(http::StatusCode::UNAUTHORIZED)
                    .set_name("TotpUnverified")
                    .set_message("user totp has not been verifieid"))
            }

            let check = transaction.execute(
                "select hash from auth_otp_codes where auth_otp_id = $1 and hash = $2 and used = false",
                &[&otp.id, &value]
            ).await?;

            if check == 0 {
                return Err(error::Error::new()
                    .set_status(http::StatusCode::UNAUTHORIZED)
                    .set_name("TotpHashInvalid")
                    .set_message("given an invalid hash"));
            }

            transaction.execute(
                "update auth_otp_codes set used = true where auth_otp_id = $1 and hash = $2",
                &[&otp.id, &value]
            ).await?;
        }
    }

    transaction.execute(
        "update user_sessions set verified = true where token = $1",
        &[&session.token]
    ).await?;

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .set_message("session verified")
        .build(Some(users::find_from_id(&*conn, &session.owner).await?))
}