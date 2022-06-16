use actix_web::{Responder, http};
use lettre::{Message, Transport};
use lettre::message::Mailbox;

use crate::request::Initiator;
use crate::response;
use crate::response::json::JsonBuilder;
use crate::state;

use response::error;

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

        let to_address_result = initiator.user.email.unwrap().parse::<Mailbox>();

        if to_address_result.is_err() {
            return JsonBuilder::new(http::StatusCode::OK)
                .set_message("invalid user email")
                .build(None::<()>);
        }

        if email.has_credentials() && email.has_relay() && email.has_from() {
            let email_message = Message::builder()
                .from(email.get_from())
                .to(to_address_result.unwrap())
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