use actix_web::{web, http, Responder, HttpRequest};
use serde::Deserialize;
use lettre::message::Mailbox;

use crate::db;

use crate::response::json::JsonBuilder;
use crate::state;
use crate::request::{initiator_from_request, Initiator};
use crate::response;
use crate::email;

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
            full_name: posted.full_name,
            level: initiator.user.level,
            email: email_value,
            email_verified
        }))
}