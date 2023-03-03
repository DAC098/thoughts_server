//! handles user tags

use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;

pub mod tag_id;

use crate::db::tables::{tags, permissions};
use crate::security::{self, InitiatorLookup, Initiator};
use crate::net::http::{error, response::{self, json::JsonBuilder}};
use crate::routing;
use crate::state;
use crate::template;

/// sends back all user tags
///
/// GET /tags
/// GET /user/{user_id}/tags
pub async fn handle_get(
    req: HttpRequest,
    security: security::state::WebSecurityState,
    db: state::WebDbState,
    template: template::WebTemplateState<'_>,
    path: web::Path<routing::path::params::OptUserPath>,
) -> error::Result<impl Responder> {
    let accept_html = response::try_check_if_html_req(&req);
    let conn = &*db.get_conn().await?;
    let lookup = InitiatorLookup::from_request(&security, conn, &req).await?;

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
        if !security::permissions::has_permission(
            conn,
            &initiator.user.id,
            &permissions::rolls::USERS_ENTRIES,
            &[permissions::abilities::READ],
            Some(&user_id)
        ).await? {
            return Err(error::build::permission_denied(
                "you do not have permissions to view this users tags"
            ));
        }

        owner = user_id;
    } else {
        if !security::permissions::has_permission(
            conn,
            &initiator.user.id,
            permissions::rolls::ENTRIES,
            &[
                permissions::abilities::READ,
                permissions::abilities::READ_WRITE
            ],
            None
        ).await? {
            return Err(error::build::permission_denied(
                "you do not have permissions to read tags"
            ));
        }

        owner = initiator.user.id;
    }

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(tags::find_from_owner(conn, owner).await?))
}

#[derive(Deserialize)]
pub struct PostTagJson {
    title: String,
    color: String,
    comment: Option<String>
}

/// creates a new tag
///
/// POST /tags
///
/// only creates a single tag for the current user 
pub async fn handle_post(
    initiator: Initiator,
    db: state::WebDbState,
    posted: web::Json<PostTagJson>
) -> error::Result<impl Responder> {
    let conn = &mut *db.get_conn().await?;
    let transaction = conn.transaction().await?;

    if !security::permissions::has_permission(
        &transaction,
        &initiator.user.id,
        permissions::rolls::ENTRIES,
        &[permissions::abilities::READ_WRITE],
        None
    ).await? {
        return Err(error::build::permission_denied(
            "you do not have permission to create tags"
        ));
    }

    let result = transaction.query_one(
        "insert into tags (title, color, comment, owner) values ($1, $2, $3, $4) returning id",
        &[&posted.title, &posted.color, &posted.comment, &initiator.user.id]
    ).await?;

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(tags::Tag {
            id: result.get(0),
            title: posted.title.clone(),
            color: posted.color.clone(),
            comment: posted.comment.clone(),
            owner: initiator.user.id
        }))
}
