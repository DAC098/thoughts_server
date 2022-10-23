use actix_web::HttpRequest;
use actix_web::{web, http, Responder};
use serde::Deserialize;

use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::{request, security, db, routing};
use crate::routing::path;
use crate::state;

pub mod permissions;
pub mod users;

pub async fn handle_get(
    req: HttpRequest,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
    path: web::Path<path::params::GroupPath>
) -> error::Result<impl Responder> {
    let conn = &*db.pool.get().await?;
    let accept_html = response::try_check_if_html_req(&req);
    let initiator_opt = request::initiator_from_request(conn, &req).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(
                &template, 
                Some(initiator_opt.unwrap().user)
            )?)
        } else {
            Ok(response::redirect_to_login(&req))
        }
    } else if initiator_opt.is_none() {
        Err(error::ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();

        if !security::permissions::has_permission(
            conn, 
            &initiator.user.id, 
            db::permissions::rolls::GROUPS, 
            &[
                db::permissions::abilities::READ,
                db::permissions::abilities::READ_WRITE
            ],
            None
        ).await? {
            return Err(error::ResponseError::PermissionDenied(
                "you do not have permissions to read groups".into()
            ))
        }

        let group = match db::groups::find_id(conn, &path.group_id).await? {
            Some(group) => group,
            None => {
                return Err(error::ResponseError::GroupNotFound(path.group_id))
            }
        };

        JsonBuilder::new(http::StatusCode::OK)
            .build(Some(group))
    }
}

#[derive(Deserialize)]
pub struct UpdateGroup {
    name: String
}

pub async fn handle_put(
    initiator: request::Initiator,
    db: state::WebDbState,
    path: web::Path<path::params::GroupPath>,
    posted: web::Json<UpdateGroup>,
) -> error::Result<impl Responder> {
    let conn = &mut *db.pool.get().await?;
    let posted = posted.into_inner();

    if !security::permissions::has_permission(
        conn, 
        &initiator.user.id, 
        db::permissions::rolls::GROUPS, 
        &[
            db::permissions::abilities::READ_WRITE
        ],
        None
    ).await? {
        return Err(error::ResponseError::PermissionDenied(
            "you do not have permissions to write groups".into()
        ))
    }

    let transaction = conn.transaction().await?;

    let mut group = match db::groups::find_id(&transaction, &path.group_id).await? {
        Some(g) => g,
        None => {
            return Err(error::ResponseError::GroupNotFound(path.group_id))
        }
    };

    let _result = transaction.execute(
        "update groups set name = $2 where id = $1",
        &[&path.group_id, &posted.name]
    ).await?;

    transaction.commit().await?;

    group.name = posted.name;

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(group))
}

pub async fn handle_delete(
    initiator: request::Initiator,
    db: state::WebDbState,
    path: web::Path<routing::path::params::GroupPath>
) -> error::Result<impl Responder> {
    let conn = &mut *db.pool.get().await?;

    if !security::permissions::has_permission(
        conn, 
        &initiator.user.id, 
        db::permissions::rolls::GROUPS, 
        &[
            db::permissions::abilities::READ_WRITE
        ],
        None
    ).await? {
        return Err(error::ResponseError::PermissionDenied(
            "you do not have permission to write groups".into()
        ));
    }

    let transaction = conn.transaction().await?;

    let _group_check = match db::groups::find_id(&transaction, &path.group_id).await? {
        Some(group) => group,
        None => {
            return Err(error::ResponseError::GroupNotFound(path.group_id))
        }
    };

    let _subject_permissions = transaction.execute(
        "delete from permissions where subject_type = 'groups' and subject_id = $1",
        &[&path.group_id]
    ).await?;

    let _resource_permissions = transaction.execute(
        "delete from permissions where resource_type = 'groups' and resource_id = $1",
        &[&path.group_id]
    ).await?;

    let _group_users = transaction.execute(
        "delete from group_users where group_id = $1",
        &[&path.group_id]
    ).await?;

    let _group = transaction.execute(
        "delete from groups where id = $1",
        &[&path.group_id]
    ).await?;

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(None::<()>)
}