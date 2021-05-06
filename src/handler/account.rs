use actix_web::{web, http, Responder, HttpRequest};
use actix_session::{Session};
use serde::{Deserialize};

use crate::error;
use crate::state;
use crate::request::from;
use crate::response;

pub async fn handle_get_account(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>
) -> error::Result<impl Responder> {
    let accept_html = response::check_if_html_req(&req, true)?;
    let conn = &app.get_conn().await?;
    let initiator_opt = from::get_initiator(conn, &session).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(Some(initiator_opt.unwrap().user)))
        } else {
            Ok(response::redirect_to_path("/auth/login"))
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
    username: Option<String>,
    full_name: Option<String>,
    email: Option<String>
}

pub async fn handle_put_account(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    posted: web::Json<PutAccountJson>,
) -> error::Result<impl Responder> {
    let conn = &mut *app.get_conn().await?;
    let mut arg_count: u32 = 2;
    let mut set_fields: Vec<String> = vec!();
    let mut query_slice: Vec<&(dyn tokio_postgres::types::ToSql + std::marker::Sync)> = vec!(initiator.user.get_id_ref());

    if let Some(username) = posted.username.as_ref() {
        set_fields.push(format!("username = ${}", arg_count));
        arg_count += 1;
        query_slice.push(username);
    }

    if let Some(full_name) = posted.full_name.as_ref() {
        set_fields.push(format!("full_name = ${}", arg_count));
        arg_count += 1;
        query_slice.push(full_name);
    }

    if let Some(email) = posted.email.as_ref() {
        set_fields.push(format!("email = ${}", arg_count));
        query_slice.push(email);
    }

    let query_str = format!(r#"
        update users
        set {}
        where id = $1
    "#, set_fields.join(", "));

    let transaction = conn.transaction().await?;

    let _result = transaction.execute(query_str.as_str(), &query_slice[..]).await?;

    transaction.commit().await?;
    
    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::<Option<()>>::build(
            "successful",
            None
        )
    ))
}