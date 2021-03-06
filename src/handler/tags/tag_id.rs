use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;

use crate::db;

use crate::request::{initiator_from_request, Initiator};
use crate::response;
use crate::response::json::JsonBuilder;
use crate::state;
use crate::security;

use response::error;

#[derive(Deserialize)]
pub struct TagIdPath {
    tag_id: i32
}

pub async fn handle_get(
    req: HttpRequest,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
    path: web::Path<TagIdPath>,
) -> error::Result<impl Responder> {
    let accept_html = response::try_check_if_html_req(&req);
    let conn = &*db.get_conn().await?;
    let initiator_opt = initiator_from_request(conn, &req).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(initiator_opt.unwrap().user))?)
        } else {
            let redirect = format!("/auth/login?jump_to=/tags/{}", path.tag_id);
            Ok(response::redirect_to_path(redirect.as_str()))
        }
    } else if initiator_opt.is_none() {
        Err(error::ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();

        if let Some(tag) = db::tags::find_from_id(conn, path.tag_id).await? {
            if tag.owner != initiator.user.id {
                Err(error::ResponseError::PermissionDenied(
                    format!("you do not have permission to view this tag. id: {}", tag.id)
                ))
            } else {
                JsonBuilder::new(http::StatusCode::OK)
                    .build(Some(tag))
            }
        } else {
            Err(error::ResponseError::TagNotFound(path.tag_id))
        }
    }
}

#[derive(Deserialize)]
pub struct PutTagJson {
    title: String,
    color: String,
    comment: Option<String>
}

pub async fn handle_put(
    initiator: Initiator,
    db: state::WebDbState,
    path: web::Path<TagIdPath>,
    posted: web::Json<PutTagJson>,
) -> error::Result<impl Responder> {
    let conn = &mut *db.get_conn().await?;
    security::assert::is_owner_for_tag(conn, &path.tag_id, &initiator.user.id).await?;

    let transaction = conn.transaction().await?;
    let _result = transaction.execute(
        "update tags set title = $1, color = $2, comment = $3 where id = $4",
        &[&posted.title, &posted.color, &posted.comment, &path.tag_id]
    ).await?;

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(db::tags::Tag {
            id: path.tag_id,
            title: posted.title.clone(),
            color: posted.color.clone(),
            comment: posted.comment.clone(),
            owner: initiator.user.id
        }))
}

pub async fn handle_delete(
    initiator: Initiator,
    db: state::WebDbState,
    path: web::Path<TagIdPath>,
) -> error::Result<impl Responder> {
    let conn = &mut *db.get_conn().await?;
    security::assert::is_owner_for_tag(conn, &path.tag_id, &initiator.user.id).await?;

    let transaction = conn.transaction().await?;

    let _entries_tags = transaction.execute(
        "delete from entries2tags where tag = $1",
        &[&path.tag_id]
    ).await?;

    let _tags = transaction.execute(
        "delete from tags where id = $1",
        &[&path.tag_id]
    ).await?;

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(None::<()>)
}