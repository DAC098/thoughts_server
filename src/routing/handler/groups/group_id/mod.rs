use actix_web::HttpRequest;
use actix_web::{web, http, Responder};
use serde::{Serialize, Deserialize};

use crate::db::tables::{
    permissions,
    groups
};
use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::security::{self, InitiatorLookup, Initiator};
use crate::components;
use crate::routing::{self, path};
use crate::state;

#[derive(Serialize)]
struct GroupUser {
    id: i32,
    username: String
}

#[derive(Serialize)]
struct GroupPermission {
    roll: String,
    ability: String,
    resource_table: Option<String>,
    resource_id: Option<i32>
}

#[derive(Serialize)]
struct GroupData {
    id: i32,
    name: String,
    users: Vec<GroupUser>,
    permissions: Vec<GroupPermission>
}

pub async fn handle_get(
    req: HttpRequest,
    security: security::state::WebSecurityState,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
    path: web::Path<path::params::GroupPath>
) -> error::Result<impl Responder> {
    let conn = &*db.pool.get().await?;
    let accept_html = response::try_check_if_html_req(&req);
    let lookup = InitiatorLookup::from_request(&security, conn, &req).await?;

    if accept_html {
        return if lookup.is_some() {
            Ok(response::respond_index_html(
                &template, 
                Some(lookup.unwrap().user)
            )?)
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
            "you do not have permissions to read groups"
        ))
    }

    let group = match groups::find_id(conn, &path.group_id).await? {
        Some(group) => group,
        None => {
            return Err(error::build::group_not_found(&path.group_id))
        }
    };

    let (attached_users, permissions) = futures_util::future::try_join(
        conn.query(
            "\
            select users.id, \
                    username \
            from users \
            join group_users on \
                users.id = group_users.users_id \
            where group_users.group_id = $1",
            &[&path.group_id]
        ),
        conn.query(
            "\
            select roll, \
                    ability, \
                    resource_table, \
                    resource_id \
            from permissions \
            where subject_table = 'groups' and \
                    subject_id = $1",
            &[&path.group_id]
        )
    ).await?;

    let mut rtn = GroupData {
        id: group.id,
        name: group.name,
        users: Vec::with_capacity(attached_users.len()),
        permissions: Vec::with_capacity(permissions.len())
    };

    for row in attached_users {
        rtn.users.push(GroupUser {
            id: row.get(0),
            username: row.get(1)
        });
    }

    for row in permissions {
        rtn.permissions.push(GroupPermission { 
            roll: row.get(0),
            ability: row.get(1),
            resource_table: row.get(2),
            resource_id: row.get(3)
        })
    }

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(rtn))
}

#[derive(Deserialize)]
pub struct UpdateGroup {
    name: String,
    users: Option<Vec<i32>>,
    permissions: Option<Vec<security::permissions::PermissionJson>>
}

pub async fn handle_put(
    initiator: Initiator,
    db: state::WebDbState,
    path: web::Path<path::params::GroupPath>,
    posted: web::Json<UpdateGroup>,
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
            "you do not have permissions to write groups"
        ))
    }

    let transaction = conn.transaction().await?;

    let _group = match groups::find_id(&transaction, &path.group_id).await? {
        Some(g) => g,
        None => {
            return Err(error::build::group_not_found(&path.group_id))
        }
    };

    {
        let _result = transaction.execute(
            "update groups set name = $2 where id = $1",
            &[&path.group_id, &posted.name]
        ).await?;
    }

    if let Some(permissions) = posted.permissions {
        let result = security::permissions::update_subject_permissions(
            &transaction,
            permissions::tables::GROUPS,
            &path.group_id,
            permissions
        ).await?;

        if result.is_some() {
            return JsonBuilder::new(http::StatusCode::BAD_REQUEST)
                .set_error("Validation")
                .set_message("some of the permissions provided are invalid")
                .build(result);
        }
    }

    if let Some(users) = posted.users {
        let result = components::groups::update_group_users(
            &transaction, 
            &path.group_id, 
            users
        ).await?;
    
        if result.is_some() {
            return JsonBuilder::new(http::StatusCode::BAD_REQUEST)
                .set_error("Validation")
                .set_message("some id's provided are not valid user ids")
                .build(result);
        }
    }

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(None::<()>)
}

pub async fn handle_delete(
    initiator: security::Initiator,
    db: state::WebDbState,
    path: web::Path<routing::path::params::GroupPath>
) -> error::Result<impl Responder> {
    let conn = &mut *db.pool.get().await?;

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

    let _group_check = match groups::find_id(&transaction, &path.group_id).await? {
        Some(group) => group,
        None => {
            return Err(error::build::group_not_found(&path.group_id))
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