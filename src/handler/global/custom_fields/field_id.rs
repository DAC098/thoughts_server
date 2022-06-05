use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;

use tlib::db;

use crate::request::{initiator_from_request, Initiator};
use crate::response;
use crate::response::json::JsonBuilder;
use crate::state;
use crate::security;
use crate::getters;

use response::error::{Result, ResponseError};

#[derive(Deserialize)]
pub struct FieldPath {
    field_id: i32
}

pub async fn handle_get(
    req: HttpRequest,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
    path: web::Path<FieldPath>,
) -> Result<impl Responder> {
    let conn = &*db.get_conn().await?;
    let initiator_opt = initiator_from_request(conn, &req).await?;
    let accept_html = response::try_check_if_html_req(&req);

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(initiator_opt.unwrap().user))?)
        } else {
            Ok(response::redirect_to_login(&req))
        }
    } else if initiator_opt.is_none() {
        Err(ResponseError::Session)
    } else {
        JsonBuilder::new(http::StatusCode::OK)
            .build(Some(getters::global_custom_fields::get_via_id(conn, &path.field_id).await?))
    }
}

#[derive(Deserialize)]
pub struct PutGlobalCustomFieldJson {
    name: String,
    comment: Option<String>,
    config: db::custom_fields::CustomFieldType
}

pub async fn handle_put(
    initiator: Initiator,
    db: state::WebDbState,
    posted: web::Json<PutGlobalCustomFieldJson>,
    path: web::Path<FieldPath>,
) -> Result<impl Responder> {
    security::assert::is_admin(&initiator)?;

    let posted = posted.into_inner();
    let conn = &mut *db.get_conn().await?;

    let _original = getters::global_custom_fields::get_via_id(conn, &path.field_id).await?;

    let transaction = conn.transaction().await?;

    let json = serde_json::to_value(posted.config.clone())?;
    transaction.execute(
        "\
        update global_custom_fields \
        set name = $1 \
            comment = $2 \
            config = $3 \
        where id = $4",
        &[
            &posted.name,
            &posted.comment,
            &json
        ]
    ).await?;

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(db::global_custom_fields::GlobalCustomField {
            id: path.field_id,
            name: posted.name,
            comment: posted.comment,
            config: posted.config
        }))
}

pub async fn handle_delete(
    initiator: Initiator,
    db: state::WebDbState,
    path: web::Path<FieldPath>,
) -> Result<impl Responder> {
    security::assert::is_admin(&initiator)?;

    let conn = &mut *db.get_conn().await?;

    let _original = getters::global_custom_fields::get_via_id(conn, &path.field_id).await?;

    let transaction = conn.transaction().await?;
    transaction.execute(
        "delete from global_custom_fields where id = $1",
        &[&path.field_id]
    ).await?;

    transaction.commit().await?;
    
    JsonBuilder::new(http::StatusCode::OK)
        .build(None::<()>)
}