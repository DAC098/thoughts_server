use actix_web::{web, http, Responder};
use serde::Deserialize;

use crate::db;

use crate::state;
use crate::net::http::error;
use crate::net::http::response::json::JsonBuilder;
use crate::security::Initiator;
use crate::security;
use crate::util;

#[derive(Deserialize)]
pub struct EntryCommentPath {
    user_id: Option<i32>,
    entry_id: i32,
    comment_id: i32,
}

#[derive(Deserialize)]
pub struct PutEntryComment {
    comment: String
}

pub async fn handle_put(
    initiator: Initiator,
    db: state::WebDbState,
    path: web::Path<EntryCommentPath>,
    posted: web::Json<PutEntryComment>,
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

    if let Some(original) = db::entry_comments::find_from_id(&transaction, &path.comment_id).await? {
        if original.owner != owner {
            return Err(error::build::permission_denied(
                format!("you are not the owner of this comment. you cannot modify another users comment")
            ));
        }

        let now = util::time::now_utc();

        transaction.execute(
            "\
            update entry_comments \
            set comment = $2, \
                updated = $3 \
            where id = $1",
            &[&original.id, &posted.comment, &now]
        ).await?;

        transaction.commit().await?;

        JsonBuilder::new(http::StatusCode::OK)
            .build(Some(db::composed::ComposedEntryComment {
                user: initiator.into(),
                comment: db::entry_comments::EntryComment {
                    id: original.id,
                    entry: original.entry,
                    owner: original.owner,
                    comment: posted.comment,
                    created: original.created,
                    updated: Some(now)
                }
            }))
    } else {
        Err(error::build::entry_comment_not_found(&path.comment_id))
    }
}

