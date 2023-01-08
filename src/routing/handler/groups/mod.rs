use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;

use crate::db::tables::{
    permissions,
    groups,
};
use crate::components;
use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::state;
use crate::security::{self, InitiatorLookup};
use crate::template;

pub mod group_id;

pub async fn handle_get(
    req: HttpRequest,
    security: security::state::WebSecurityState,
    db: state::WebDbState,
    template: template::WebTemplateState<'_>
) -> std::result::Result<impl Responder, error::Error> {
    let conn = &*db.pool.get().await?;
    let accept_html = response::try_check_if_html_req(&req);
    let lookup = InitiatorLookup::from_request(&security, conn, &req).await?;

    if accept_html {
        return if lookup.is_valid() {
            Ok(response::respond_index_html(
                &template, 
                Some(lookup.unwrap().user))?)
        } else {
            Ok(response::redirect_to_login(&req))
        }
    }

    let initiator = lookup.try_into()?;

    if !security::permissions::has_permission(
        conn, 
        &initiator.user.id, 
        permissions::rolls::GROUPS, 
        &[
            permissions::abilities::READ,
            permissions::abilities::READ_WRITE
        ],
        None
    ).await? {
        return Err(error::build::permission_denied(
            "you do not have permission to read groups"
        ))
    }

    let groups = groups::get_all(conn).await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(groups))
        .map_err(|e| e.into())
}

#[derive(Deserialize)]
pub struct NewGroupJson {
    name: String,
    users: Option<Vec<i32>>,
    permissions: Option<Vec<security::permissions::PermissionJson>>
}

pub async fn handle_post(
    initiator: security::Initiator,
    db: state::WebDbState,
    posted: web::Json<NewGroupJson>
) -> error::Result<impl Responder> {
    let conn = &mut *db.pool.get().await?;
    let posted = posted.into_inner();
    
    if !security::permissions::has_permission(
        conn, 
        &initiator.user.id,
        permissions::rolls::GROUPS,
        &[
            permissions::abilities::READ_WRITE
        ], 
        None
    ).await? {
        return Err(error::build::permission_denied(
            "you do not have permission to write groups"
        ));
    }

    let transaction = conn.transaction().await?;

    let group_check = transaction.execute(
        "select id from groups where name = $1",
        &[&posted.name]
    ).await?;

    if group_check != 0 {
        return Err(error::build::group_already_exists(
            posted.name.clone()
        ));
    }

    let result = transaction.query_one(
        "insert into groups (name) values ($1) returning id",
        &[&posted.name]
    ).await?;

    let group_id: i32 = result.get(0);

    if let Some(users) = posted.users {
        let result = components::groups::update_group_users(
            &transaction, 
            &group_id, 
            users
        ).await?;

        if result.is_some() {
            return JsonBuilder::new(http::StatusCode::BAD_REQUEST)
                .set_error("Validation")
                .set_message("some id's provided are not valid user ids")
                .build(result)
        }
    }

    if let Some(permissions) = posted.permissions {
        let result = security::permissions::update_subject_permissions(
            &transaction, 
            permissions::tables::GROUPS, 
            &group_id, 
            permissions
        ).await?;

        if result.is_some() {
            return JsonBuilder::new(http::StatusCode::BAD_REQUEST)
                .set_error("Validation")
                .set_message("some of the permissions provided are invalid")
                .build(result);
        }
    }

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(group_id))
}