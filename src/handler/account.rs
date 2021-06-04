use actix_web::{web, http, Responder, HttpRequest};
use actix_session::{Session};
use serde::{Deserialize};
use lettre::{Message, Transport};
use lettre::message::{Mailbox};

use crate::state;
use crate::request::from;
use crate::response;
use crate::email;
use crate::util;
use crate::db;

use response::error;

pub async fn handle_get(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>
) -> error::Result<impl Responder> {
    let accept_html = response::check_if_html_req(&req, true)?;
    let conn = &*app.as_ref().get_conn().await?;
    let initiator_opt = from::get_initiator(conn, &session).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(Some(initiator_opt.unwrap().user)))
        } else {
            Ok(response::redirect_to_path("/auth/login?jump_to=/account"))
        }
    } else if initiator_opt.is_none() {
        Err(error::ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();

        Ok(response::json::respond_json(
            http::StatusCode::OK,
            response::json::MessageDataJSON::build(
                "successful",
                initiator.user
            )
        ))
    }
}

#[derive(Deserialize)]
pub struct PutAccountJson {
    username: String,
    full_name: Option<String>,
    email: String
}

pub async fn handle_put(
    initiator: from::Initiator,
    app_data: web::Data<state::AppState>,
    posted_json: web::Json<PutAccountJson>,
) -> error::Result<impl Responder> {
    let app = app_data.into_inner();
    let posted = posted_json.into_inner();
    let conn = &mut *app.get_conn().await?;
    let mut email_value: Option<String> = None;
    let mut email_verified: bool = false;
    let mut to_mailbox: Option<Mailbox> = None;

    if app.email.enabled {
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
        r#"
        update users
        set username = $2,
            full_name = $3,
            email = $4,
            email_verified = $5
        where id = $1
        "#,
        &[
            &initiator.user.id,
            &posted.username,
            &posted.full_name,
            &email_value,
            &email_verified
        ]
    ).await?;

    transaction.commit().await?;

    if app.email.enabled && !email_verified {
        let mut rand_bytes: [u8; 32] = [0; 32];
        openssl::rand::rand_bytes(&mut rand_bytes)?;
        let hex_str = util::hex_string(&rand_bytes)?;
        let issued = util::time::now();

        conn.execute(
            r#"
            insert into email_verifications (owner, key_id, issued) values
            ($1, $2, $3)
            on conflict on constraint email_verifications_pkey do update
            set key_id = excluded.key_id,
                issued = excluded.issued
            "#,
            &[&initiator.user.id, &hex_str, &issued]
        ).await?;

        let transport = app.email.get_transport()?;
        let email_message = Message::builder()
            .from(app.email.get_from())
            .to(to_mailbox.unwrap())
            .subject("Verify Changed Email")
            .multipart(email::message_body::verify_email_body(
                app.info.url_origin(), hex_str
            ))?;

        transport.send(&email_message)?;
    }
    
    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful",
            db::users::User {
                id: initiator.user.id,
                username: posted.username,
                full_name: posted.full_name,
                level: initiator.user.level,
                email: email_value,
                email_verified: email_verified
            }
        )
    ))
}