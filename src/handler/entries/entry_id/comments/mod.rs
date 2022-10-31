use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;

use crate::db;

pub mod comment_id;

use crate::state;
use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::security::{initiator_from_request, Initiator};
use crate::security;
use crate::util;

#[derive(Deserialize)]
pub struct EntryPath {
    user_id: Option<i32>,
    entry_id: i32
}

pub async fn handle_get(
    req: HttpRequest,
    security: state::WebSecurityState,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
    path: web::Path<EntryPath>,
) -> error::Result<impl Responder> {
    let path = path.into_inner();
    let conn = &*db.get_conn().await?;
    let accept_html = response::try_check_if_html_req(&req);
    let initiator = initiator_from_request(&security, conn, &req).await?;

    if accept_html {
        if initiator.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(initiator.unwrap().user))?)
        } else {
            Ok(response::redirect_to_login(&req))
        }
    } else if initiator.is_none() {
        Err(error::ResponseError::Session)
    } else {
        let initiator = initiator.unwrap();
        let owner: i32;

        if let Some(user_id) = path.user_id {
            security::assert::permission_to_read(conn, &initiator.user.id, &user_id).await?;
            owner = user_id;
        } else {
            owner = initiator.user.id;
        }

        security::assert::is_owner_of_entry(conn, &owner, &path.entry_id).await?;

        JsonBuilder::new(http::StatusCode::OK)
            .build(Some(db::composed::ComposedEntryComment::find_from_entry(
                conn,
                &path.entry_id
            ).await?))
    }
}

#[derive(Deserialize)]
pub struct PostEntryComment {
    comment: String
}

pub async fn handle_post(
    initiator: Initiator,
    db: state::WebDbState,
    path: web::Path<EntryPath>,
    posted: web::Json<PostEntryComment>,
) -> error::Result<impl Responder> {
    let initiator = initiator.into_user();
    let posted = posted.into_inner();
    let path = path.into_inner();
    let conn = &mut *db.get_conn().await?;
    let owner: i32;

    if let Some(user_id) = path.user_id {
        security::assert::permission_to_read(conn, &initiator.id, &user_id).await?;
        owner = user_id;
    } else {
        owner = initiator.id;
    }

    security::assert::is_owner_of_entry(conn, &owner, &path.entry_id).await?;

    let transaction = conn.transaction().await?;
    let now = util::time::now_utc();
    let record = transaction.query_one(
        "\
        insert into entry_comments (entry, owner, comment, created) values\
        ($1, $2, $3, $4)\
        returning id",
        &[
            &path.entry_id,
            &owner,
            &posted.comment,
            &now
        ]
    ).await?;

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(db::composed::ComposedEntryComment {
            user: initiator.into(),
            comment: db::entry_comments::EntryComment {
                id: record.get(0),
                entry: path.entry_id,
                owner: owner,
                comment: posted.comment,
                created: now,
                updated: None
            }
        }))
}