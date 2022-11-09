use actix_web::{web, http, Responder, HttpRequest};
use serde::Deserialize;
use lettre::message::Mailbox;

use crate::db;

use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::state;
use crate::security::{self, initiator, Initiator};
use crate::email;

pub async fn handle_get(
    req: HttpRequest,
    security: security::state::WebSecurityState,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
) -> error::Result<impl Responder> {
    let accept_html = response::try_check_if_html_req(&req);
    let conn = &*db.get_conn().await?;
    let lookup = initiator::from_request(&security, conn, &req).await?;

    if accept_html {
        return if lookup.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(lookup.unwrap().user))?)
        } else {
            Ok(response::redirect_to_path("/auth/login?jump_to=/account"))
        }
    }
    
    let initiator = lookup.try_into()?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(initiator.user))
}

#[derive(Deserialize)]
pub struct PutAccountJson {
    username: String,
    email: String
}

pub async fn handle_put(
    initiator: Initiator,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
    email: state::WebEmailState,
    server_info: state::WebServerInfoState,
    posted_json: web::Json<PutAccountJson>,
) -> error::Result<impl Responder> {
    let posted = posted_json.into_inner();
    let conn = &mut *db.get_conn().await?;
    let mut email_value: Option<String> = None;
    let mut email_verified: bool = false;
    let mut to_mailbox: Option<Mailbox> = None;

    if email.is_enabled() {
        let check = email::validate_new_email(&*conn, &posted.email, &initiator.user.id).await?;

        if let Some(original_email) = initiator.user.email {
            if original_email == posted.email {
                email_verified = initiator.user.email_verified;
            }
        }

        email_value = Some(posted.email);
        to_mailbox = Some(Mailbox::new(None, check));
    }

    let transaction = conn.transaction().await?;

    let _result = transaction.execute(
        "\
        update users \
        set username = $2, \
            email = $3, \
            email_verified = $4 \
        where id = $1",
        &[
            &initiator.user.id,
            &posted.username,
            &email_value,
            &email_verified
        ]
    ).await?;

    if email.is_enabled() && !email_verified {
        email::send_verify_email(
            &transaction,
            &server_info,
            &email,
            &template,
            &initiator.user.id, 
            to_mailbox.unwrap()
        ).await?;
    }

    transaction.commit().await?;
    
    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(db::users::User {
            id: initiator.user.id,
            username: posted.username,
            level: initiator.user.level,
            email: email_value,
            email_verified
        }))
}