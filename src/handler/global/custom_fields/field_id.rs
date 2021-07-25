use actix_web::{web, http, HttpRequest, Responder};
use actix_session::{Session};
use serde::{Deserialize};

use crate::request::from;
use crate::response;
use crate::state;
use crate::json;
use crate::db;
use crate::security;
use crate::getters;

use response::error::{Result, ResponseError};

#[derive(Deserialize)]
pub struct FieldPath {
    field_id: i32
}

pub async fn handle_get(
    req: HttpRequest,
    session: Session,
    app_wrapper: web::Data<state::AppState>,
    path: web::Path<FieldPath>,
) -> Result<impl Responder> {
    let app = app_wrapper.into_inner();
    let conn = &*app.get_conn().await?;
    let initiator_opt = from::get_initiator(conn, &session).await?;
    let accept_html = response::check_if_html_req(&req, true)?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(Some(initiator_opt.unwrap().user)))
        } else {
            Ok(response::redirect_to_login(&req))
        }
    } else if initiator_opt.is_none() {
        Err(ResponseError::Session)
    } else {
        Ok(response::json::respond_json(
            http::StatusCode::OK,
            response::json::MessageDataJSON::build(
                "successful",
                getters::global_custom_fields::get_via_id(conn, path.field_id).await?
            )
        ))
    }
}

#[derive(Deserialize)]
pub struct PutGlobalCustomFieldJson {
    name: String,
    comment: Option<String>,
    config: db::custom_fields::CustomFieldType
}

pub async fn handle_put(
    initiator: from::Initiator,
    app_wrapper: web::Data<state::AppState>,
    posted_wrapper: web::Json<PutGlobalCustomFieldJson>,
    path: web::Path<FieldPath>,
) -> Result<impl Responder> {
    security::assert::is_admin(&initiator)?;

    let app = app_wrapper.into_inner();
    let posted = posted_wrapper.into_inner();
    let conn = &mut *app.get_conn().await?;

    let _original = getters::global_custom_fields::get_via_id(conn, path.field_id).await?;

    let transaction = conn.transaction().await?;

    let json = serde_json::to_value(posted.config.clone())?;
    transaction.execute(
        r#"\
        update global_custom_fields \
        set name = $1 \
            comment = $2 \
            config = $3 \
        where id = $4"#,
        &[
            &posted.name,
            &posted.comment,
            &json
        ]
    ).await?;

    transaction.commit().await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful",
            json::GlobalCustomFieldJson {
                id: path.field_id,
                name: posted.name,
                comment: posted.comment,
                config: posted.config
            }
        )
    ))
}

pub async fn handle_delete(
    initiator: from::Initiator,
    app_wrapper: web::Data<state::AppState>,
    path: web::Path<FieldPath>,
) -> Result<impl Responder> {
    security::assert::is_admin(&initiator)?;

    let app = app_wrapper.into_inner();
    let conn = &mut *app.get_conn().await?;

    let _original = getters::global_custom_fields::get_via_id(conn, path.field_id).await?;

    let transaction = conn.transaction().await?;
    transaction.execute(
        "delete from global_custom_fields where id = $1",
        &[&path.field_id]
    ).await?;

    transaction.commit().await?;
    
    Ok(response::json::respond_okay())
}