use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;

use crate::db;

pub mod tag_id;

use crate::security::{initiator, Initiator};
use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::state;
use crate::security;

#[derive(Deserialize)]
pub struct TagsPath {
    user_id: Option<i32>
}

pub async fn handle_get(
    req: HttpRequest,
    security: security::state::WebSecurityState,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
    path: web::Path<TagsPath>,
) -> error::Result<impl Responder> {
    let accept_html = response::try_check_if_html_req(&req);
    let conn = &*db.get_conn().await?;
    let lookup = initiator::from_request(&security, conn, &req).await?;

    if accept_html {
        return if lookup.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(lookup.unwrap().user))?)
        } else {
            Ok(response::redirect_to_path("/auth/login?jump_to=/tags"))
        }
    }

    let initiator = lookup.try_into()?;
    let owner: i32;

    if let Some(user_id) = path.user_id {
        security::assert::permission_to_read(conn, &initiator.user.id, &user_id).await?;
        owner = user_id;
    } else {
        owner = initiator.user.id;
    }

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(db::tags::find_from_owner(conn, owner).await?))
}

#[derive(Deserialize)]
pub struct PostTagJson {
    title: String,
    color: String,
    comment: Option<String>
}

pub async fn handle_post(
    initiator: Initiator,
    db: state::WebDbState,
    posted: web::Json<PostTagJson>
) -> error::Result<impl Responder> {
    let conn = &mut *db.get_conn().await?;
    let transaction = conn.transaction().await?;

    let result = transaction.query_one(
        "insert into tags (title, color, comment, owner) values ($1, $2, $3, $4) returning id",
        &[&posted.title, &posted.color, &posted.comment, &initiator.user.id]
    ).await?;

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(db::tags::Tag {
            id: result.get(0),
            title: posted.title.clone(),
            color: posted.color.clone(),
            comment: posted.comment.clone(),
            owner: initiator.user.id
        }))
}