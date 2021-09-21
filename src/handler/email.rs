use actix_web::{Responder};
use lettre::{Message, Transport};
use lettre::message::{Mailbox};

use crate::request::from;
use crate::response;
use crate::state;

use response::error;

pub async fn handle_get(
    initiator: from::Initiator,
    email: state::WebEmailState,
) -> error::Result<impl Responder> {
    if email.is_enabled() {
        if initiator.user.email.is_none() {
            return Ok(response::json::respond_message("no email specified"));
        }

        if !initiator.user.email_verified {
            return Ok(response::json::respond_message("unverified email"));
        }

        let to_address_result = initiator.user.email.unwrap().parse::<Mailbox>();

        if to_address_result.is_err() {
            return Ok(response::json::respond_message("invalid user email"));
        }

        if email.can_get_transport() && email.has_from() {
            let email_message = Message::builder()
            .from(email.get_from().unwrap())
            .to(to_address_result.unwrap())
            .subject("test email")
            .body("test email being sent".to_owned())?;

            let transport = email.get_transport()?;
            transport.send(&email_message)?;

            Ok(response::json::respond_message("email sent"))
        } else {
            Ok(response::json::respond_message("missing email information"))
        }
    } else {
        Ok(response::json::respond_message("email disabled"))
    }
}