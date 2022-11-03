use std::collections::HashSet;

use actix_web::HttpRequest;
use actix_web::{web, http, Responder};
use serde::{Serialize, Deserialize};

use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::{security, db, routing};
use crate::routing::path;
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
    security: state::WebSecurityState,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
    path: web::Path<path::params::GroupPath>
) -> error::Result<impl Responder> {
    let conn = &*db.pool.get().await?;
    let accept_html = response::try_check_if_html_req(&req);
    let lookup = security::initiator::from_request(&security, conn, &req).await?;

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
        db::permissions::rolls::GROUPS,
        &[
            db::permissions::abilities::READ,
            db::permissions::abilities::READ_WRITE
        ],
        None
    ).await? {
        return Err(error::build::permission_denied(
            "you do not have permissions to read groups"
        ))
    }

    let group = match db::groups::find_id(conn, &path.group_id).await? {
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
    initiator: security::Initiator,
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
        return Err(error::build::permission_denied(
            "you do not have permissions to write groups"
        ))
    }

    let transaction = conn.transaction().await?;

    let _group = match db::groups::find_id(&transaction, &path.group_id).await? {
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
            db::permissions::tables::GROUPS,
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
        let users_check = transaction.query(
            "select id from users where id = any($1)",
            &[&users]
        ).await?;
        let mut id_set: HashSet<i32> = HashSet::with_capacity(users_check.len());
        let mut invalid_ids = Vec::with_capacity(users.len());
        let mut valid_ids = Vec::with_capacity(users_check.len());
    
        for row in users_check {
            let id = row.get(0);
            id_set.insert(id);
            valid_ids.push(id);
        }
    
        for id in users {
            if !id_set.contains(&id) {
                invalid_ids.push(id);
            }
        }
    
        if invalid_ids.len() > 0 {
            return JsonBuilder::new(http::StatusCode::BAD_REQUEST)
                .set_error("Validation")
                .set_message("some id's provided are not valid user ids")
                .build(Some(invalid_ids))
        }
    
        let mut query = "insert into group_users (group_id, users_id) values ".to_owned();
        let mut params = db::query::QueryParams::with_capacity(valid_ids.len() + 1);
        params.push(&path.group_id);
    
        for i in 0..valid_ids.len() {
            let key = params.push(&valid_ids[i]).to_string();
            
            if i == 0 {
                query.reserve(key.len() + 6);
            } else {
                query.reserve(key.len() + 7);
                query.push(',');
            }
    
            query.push_str("($1,$");
            query.push_str(&key);
            query.push_str(")");
        }
    
        query.push_str(" on conflict (users_id, group_id) do nothing");
    
        transaction.execute(query.as_str(), params.slice()).await?;
        transaction.execute(
            "delete from group_users where group_id = $1 and users_id <> all($2)",
            &[&path.group_id, &valid_ids]
        ).await?;
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
        db::permissions::rolls::GROUPS, 
        &[
            db::permissions::abilities::READ_WRITE
        ],
        None
    ).await? {
        return Err(error::build::permission_denied(
            "you do not have permission to write groups"
        ));
    }

    let transaction = conn.transaction().await?;

    let _group_check = match db::groups::find_id(&transaction, &path.group_id).await? {
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