use std::collections::HashSet;

use actix_web::{web, http, HttpRequest, Responder};
use serde::Serialize;

use crate::db;
use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::request;
use crate::routing::path;
use crate::security;
use crate::state;

#[derive(Serialize)]
pub struct UserItem {
    id: i32,
    username: String
}

pub async fn handle_get(
    req: HttpRequest,
    db: state::WebDbState,
    path: web::Path<path::params::GroupPath>,
) -> error::Result<impl Responder> {
    let conn = &*db.pool.get().await?;
    let accept_html = response::try_check_if_html_req(&req);
    let initiator_opt = request::initiator_from_request(conn, &req).await?;

    if accept_html {
        if initiator_opt.is_some() {
            let group_id_str = path.group_id.to_string();
            let mut redirect = "/groups/".to_owned();
            redirect.push_str(&group_id_str);

            Ok(response::redirect_to_path(&redirect))
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
                "you do not haver permissions to read groups".into()
            ))
        }

        let result = conn.query(
            "\
            select id, \
                   username \
            from users \
            join group_users on \
                users.id = group_users.users_id \
            where group_users.group_id = $1",
            &[&path.group_id]
        ).await?;
        let mut user_list = Vec::with_capacity(result.len());

        for row in result {
            user_list.push(UserItem {
                id: row.get(0),
                username: row.get(1)
            });
        }

        JsonBuilder::new(http::StatusCode::OK)
            .build(Some(user_list))
    }
}

pub async fn handle_put(
    initiator: request::Initiator,
    db: state::WebDbState,
    path: web::Path<path::params::GroupPath>,
    posted: web::Json<Vec<i32>>,
) -> error::Result<impl Responder> {
    let conn = &mut *db.pool.get().await?;
    let mut posted = posted.into_inner();

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
        ))
    }

    let transaction = conn.transaction().await?;
    posted.sort_unstable();
    
    let users_check = transaction.query(
        "select id from users where id = any($1)",
        &[&posted]
    ).await?;
    let mut id_set: HashSet<i32> = HashSet::with_capacity(users_check.len());
    let mut invalid_ids = Vec::with_capacity(posted.len());
    let mut valid_ids = Vec::with_capacity(users_check.len());

    for row in users_check {
        let id = row.get(0);
        id_set.insert(id);
        valid_ids.push(id);
    }

    for id in posted {
        if !id_set.contains(&id) {
            invalid_ids.push(id);
        }
    }

    if invalid_ids.len() > 0 {
        let mut first = true;
        let mut msg = String::new();
        msg.push_str("some id's provided are not valid user ids. ids: ");
        
        for id in invalid_ids {
            let id_str = id.to_string();

            if first {
                first = false;
            } else {
                msg.push_str(", ");
            }

            msg.push_str(&id_str);
        }

        return Err(error::ResponseError::Validation(msg))
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

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(None::<()>)
}