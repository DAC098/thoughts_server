use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;

use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::state;
use crate::db;
use crate::security;

pub mod group_id;

pub async fn handle_get(
    req: HttpRequest,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>
) -> error::Result<impl Responder> {
    let conn = &*db.pool.get().await?;
    let accept_html = response::try_check_if_html_req(&req);
    let initiator_opt = security::initiator_from_request(conn, &req).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(&template, Some(initiator_opt.unwrap().user))?)
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
                "you do not have permission to read groups".into()
            ))
        }

        let groups = db::groups::get_all(conn).await?;

        JsonBuilder::new(http::StatusCode::OK)
            .build(Some(groups))
    }
}

#[derive(Deserialize)]
pub struct NewGroupJson {
    name: String
}

pub async fn handle_post(
    initiator: security::Initiator,
    db: state::WebDbState,
    posted: web::Json<NewGroupJson>
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

    let group_check = transaction.execute(
        "select id from groups where name = $1",
        &[&posted.name]
    ).await?;

    if group_check != 0 {
        return Err(error::ResponseError::GroupAlreadyExists(
            posted.name.clone()
        ));
    }

    let result = transaction.query_one(
        "insert into groups (name) values ($1) returning id",
        &[&posted.name]
    ).await?;

    let group_id: i32 = result.get(0);

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(group_id))
}