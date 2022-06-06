use actix_web::{web, http, Responder, HttpRequest};
use serde::Deserialize;
use lettre::{Message, Transport};
use lettre::message::Mailbox;

use crate::db;

use crate::response::json::JsonBuilder;
use crate::state;
use crate::request::{initiator_from_request, Initiator};
use crate::response;
use crate::email;
use crate::util;

use response::error;

pub async fn handle_get(
    req: HttpRequest,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
) -> error::Result<impl Responder> {
    let accept_html = response::try_check_if_html_req(&req);
    let conn = &*db.get_conn().await?;
    let initiator_opt = initiator_from_request(conn, &req).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(initiator_opt.unwrap().user))?)
        } else {
            Ok(response::redirect_to_path("/auth/login?jump_to=/account"))
        }
    } else if initiator_opt.is_none() {
        Err(error::ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();

        JsonBuilder::new(http::StatusCode::OK)
            .build(Some(initiator.user))
    }
}

#[derive(Deserialize)]
pub struct PutAccountJson {
    username: String,
    full_name: Option<String>,
    email: String
}

pub async fn handle_put(
    initiator: Initiator,
    db: state::WebDbState,
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
        let to_mailbox_result = posted.email.parse::<Mailbox>();

        if to_mailbox_result.is_err() {
            return Err(error::ResponseError::Validation(
                format!("given email address is invalid. {}", posted.email)
            ));
        } else {
            to_mailbox = Some(to_mailbox_result.unwrap());
        }

        let check = conn.query(
            "select id from users where email = $1",
            &[&posted.email]
        ).await?;

        if !check.is_empty() && check[0].get::<usize, i32>(0) != initiator.user.id {
            return Err(error::ResponseError::EmailExists(posted.email));
        }

        if initiator.user.email.is_some() {
            if initiator.user.email.unwrap() == posted.email {
                email_verified = initiator.user.email_verified;
            } else {
                email_verified = false;
            }
        }

        email_value = Some(posted.email);
    }

    let transaction = conn.transaction().await?;

    let _result = transaction.execute(
        "\
        update users \
        set username = $2, \
            full_name = $3, \
            email = $4, \
            email_verified = $5 \
        where id = $1",
        &[
            &initiator.user.id,
            &posted.username,
            &posted.full_name,
            &email_value,
            &email_verified
        ]
    ).await?;

    if email.is_enabled() && !email_verified && email.can_get_transport() && email.has_from() {
        let mut rand_bytes: [u8; 32] = [0; 32];
        openssl::rand::rand_bytes(&mut rand_bytes)?;
        let hex_str = util::hex_string(&rand_bytes)?;
        let issued = util::time::now();

        transaction.execute(
            "\
            insert into email_verifications (owner, key_id, issued) values \
            ($1, $2, $3) \
            on conflict on constraint email_verifications_pkey do update \
            set key_id = excluded.key_id, \
                issued = excluded.issued",
            &[&initiator.user.id, &hex_str, &issued]
        ).await?;

        let email_message = Message::builder()
            .from(email.get_from().unwrap())
            .to(to_mailbox.unwrap())
            .subject("Verify Changed Email")
            .multipart(email::message_body::verify_email_body(
                server_info.url_origin(), hex_str
            ))?;

        email.get_transport()?.send(&email_message)?;
    }

    transaction.commit().await?;
    
    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(db::users::User {
            id: initiator.user.id,
            username: posted.username,
            full_name: posted.full_name,
            level: initiator.user.level,
            email: email_value,
            email_verified
        }))
}