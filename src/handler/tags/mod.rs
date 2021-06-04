use actix_web::{web, http, HttpRequest, Responder};
use actix_session::{Session};
use serde::{Deserialize};

pub mod tag_id;

use crate::request::from;
use crate::response;
use crate::state;
use crate::db;

use response::error;

pub async fn handle_get(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>
) -> error::Result<impl Responder> {
    let accept_html = response::check_if_html_req(&req, true)?;
    let conn = &*app.get_conn().await?;
    let initiator_opt = from::get_initiator(conn, &session).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(Some(initiator_opt.unwrap().user)))
        } else {
            Ok(response::redirect_to_path("/auth/login?jump_to=/tags"))
        }
    } else if initiator_opt.is_none() {
        Err(error::ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();

        Ok(response::json::respond_json(
            http::StatusCode::OK,
            response::json::MessageDataJSON::build(
                "successful",
                db::tags::get_via_owner(conn, initiator.user.id).await?
            )
        ))
    }
}

#[derive(Deserialize)]
pub struct PostTagJson {
    title: String,
    color: String,
    comment: Option<String>
}

pub async fn handle_post(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    posted: web::Json<PostTagJson>
) -> error::Result<impl Responder> {
    let conn = &mut *app.get_conn().await?;
    let transaction = conn.transaction().await?;

    let result = transaction.query_one(
        "insert into tags (title, color, comment, owner) values ($1, $2, $3, $4) returning id",
        &[&posted.title, &posted.color, &posted.comment, &initiator.user.id]
    ).await?;

    transaction.commit().await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful",
            db::tags::Tag {
                id: result.get(0),
                title: posted.title.clone(),
                color: posted.color.clone(),
                comment: posted.comment.clone(),
                owner: initiator.user.id
            }
        )
    ))
}