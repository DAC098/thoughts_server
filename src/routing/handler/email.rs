//! handles email related tasks
//!
//! currently more for testing purposes in trying out if email is properly
//! working

use actix_web::{Responder, http};
use lettre::{Message, Transport};
use lettre::message::Mailbox;

use crate::security::Initiator;
use crate::net::http::error;
use crate::net::http::response::json::JsonBuilder;
use crate::state;

/// sends test email to current user
///
/// GET /email
///
/// should probably be a post request. will attempt to send a simple text email
/// to the users current verified email address if email is enabled and all
/// required information for sending an email is provided
pub async fn handle_get(
    initiator: Initiator,
    email: state::WebEmailState,
) -> error::Result<impl Responder> {
    if email.is_enabled() {
        if initiator.user.email.is_none() {
            return JsonBuilder::new(http::StatusCode::OK)
                .set_message("no email specified")
                .build(None::<()>);
        }

        if !initiator.user.email_verified {
            return JsonBuilder::new(http::StatusCode::OK)
                .set_message("unverified email")
                .build(None::<()>);
        }

        let to_address = {
            let Some(user_email) = initiator.user.email else {
                return Err(error::Error::new()
                    .set_name("MissingUserEmail")
                    .set_source("initiator user email is missing when expected"));
            };

            match user_email.parse::<Mailbox>() {
                Ok(rtn) => rtn,
                Err(err) => {
                    return Err(error::Error::new()
                        .set_name("InvalidEmail")
                        .set_message("current user email is not a valid format")
                        .set_source(err));
                }
            }
        };

        if email.has_credentials() && email.has_relay() && email.has_from() {
            let email_message = Message::builder()
                .from(email.get_from())
                .to(to_address)
                .subject("test email")
                .body("test email being sent".to_owned())?;

            let transport = email.get_transport()?;
            transport.send(&email_message)?;

            JsonBuilder::new(http::StatusCode::OK)
                .set_message("email sent")
                .build(None::<()>)
        } else {
            JsonBuilder::new(http::StatusCode::OK)
                .set_message("missing email information")
                .build(None::<()>)
        }
    } else {
        JsonBuilder::new(http::StatusCode::OK)
            .set_message("email disabled")
            .build(None::<()>)
    }
}
