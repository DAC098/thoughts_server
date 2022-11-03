use lettre::{Message, Transport};
use lettre::message::{Mailbox, MultiPart};
use lettre::address::Address;
use serde_json::json;
use tokio_postgres::GenericClient;

use crate::net::http::error;
use crate::{security, util};
use crate::state::{TemplateState, EmailState, ServerInfoState};

pub fn parse_email_address(email: &str) -> error::Result<Address> {
    match email.parse::<Address>() {
        Ok(value) => Ok(value),
        Err(_error) => {
            let mut msg = String::with_capacity(email.len() + 41);
            msg.push_str("given email address is invalid. email: \"");
            msg.push_str(email);
            msg.push('"');

            Err(error::build::validation(msg))
        }
    }
}

pub async fn validate_new_email(conn: &impl GenericClient, new_email: &str, user_id: &i32) -> error::Result<Address> {
    let address = parse_email_address(new_email)?;

    if let Some(record) = conn.query_opt(
        "select id from users where email = $1",
        &[&new_email]
    ).await? {
        let record_id: i32 = record.get(0);

        if record_id != *user_id {
            return Err(error::build::email_exists(new_email));
        }
    };

    Ok(address)
}

pub async fn send_verify_email(
    conn: &impl GenericClient,
    info: &ServerInfoState,
    email: &EmailState,
    template: &TemplateState<'_>,
    owner: &i32,
    to_mailbox: Mailbox
) -> error::Result<()> {
    let issued = chrono::Utc::now();
    let verify_id = {
        let bytes = security::get_rand_bytes(32)?;
        util::hex_string(&bytes)?
    };

    conn.execute(
        "\
        insert into email_verifications (owner, key_id, issued) values \
        ($1, $2, $3) \
        on conflict on constraint email_verifications_pkey do update \
        set key_id = excluded.key_id, \
            issued = excluded.issued",
        &[owner, &verify_id, &issued]
    ).await?;

    let render_json = json!({
        "origin": info.url_origin(),
        "verify_id": verify_id
    });
    let (text_body, html_body) = template.render_email_parts("verify_email", &render_json)?;
    let message = Message::builder()
        .from(email.get_from())
        .to(to_mailbox)
        .subject("Verify Changed Email")
        .multipart(MultiPart::alternative_plain_html(text_body, html_body))?;

    email.get_transport()?.send(&message)?;

    Ok(())
}