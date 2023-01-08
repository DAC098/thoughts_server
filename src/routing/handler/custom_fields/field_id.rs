use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;

use crate::db::tables::custom_fields;
use crate::security::{self, InitiatorLookup, Initiator};
use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::state;
use crate::template;

#[derive(Deserialize)]
pub struct CustomFieldPath {
    user_id: Option<i32>,
    field_id: i32
}

pub async fn handle_get(
    req: HttpRequest,
    security: security::state::WebSecurityState,
    db: state::WebDbState,
    template: template::WebTemplateState<'_>,
    path: web::Path<CustomFieldPath>
) -> error::Result<impl Responder> {
    let accept_html = response::try_check_if_html_req(&req);
    let conn = &*db.get_conn().await?;
    let lookup = InitiatorLookup::from_request(&security, conn, &req).await?;

    if accept_html {
        return if lookup.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(lookup.unwrap().user))?)
        } else {
            let redirect = format!("/auth/login?jump_to=/custom_fields/{}", path.field_id);
            Ok(response::redirect_to_path(redirect.as_str()))
        }
    }

    let initiator = lookup.try_into()?;
    let owner: i32;

    if let Some(user_id) = path.user_id {
        security::assert::permission_to_read(conn, &initiator.user.id, &user_id).await?;
        owner = user_id;
    } else {
        owner = initiator.user.id;
    }

    if let Some(field) = custom_fields::find_from_id(conn, &path.field_id).await? {
        if field.owner == owner {
            JsonBuilder::new(http::StatusCode::OK)
                .build(Some(field))
        } else {
            Err(error::build::permission_denied(
                format!("custom field owner mis-match. requested custom field is not owned by {}", owner)
            ))
        }
    } else {
        Err(error::build::custom_field_not_found(&path.field_id))
    }
}

#[derive(Deserialize, Debug)]
pub struct PutCustomFieldJson {
    name: String,
    config: custom_fields::CustomFieldType,
    comment: Option<String>,
    order: i32
}

pub async fn handle_put(
    initiator: Initiator,
    db: state::WebDbState,
    path: web::Path<CustomFieldPath>,
    posted: web::Json<PutCustomFieldJson>,
) -> error::Result<impl Responder> {
    let conn = db.pool.get().await?;
    let posted = posted.into_inner();

    security::assert::is_owner_for_custom_field(&*conn, &path.field_id, &initiator.user.id).await?;

    let config_json = serde_json::to_value(posted.config.clone())?;
    let _result = conn.execute(
        "\
        update custom_fields \
        set name = $1, \
            config = $2, \
            comment = $3, \
            \"order\" = $4 \
        where id = $5 \
        returning name, comment",
        &[
            &posted.name,
            &config_json,
            &posted.comment,
            &posted.order,
            &path.field_id
        ]
    ).await?;

    let rtn = custom_fields::CustomField {
        id: path.field_id,
        name: posted.name,
        config: posted.config,
        comment: posted.comment,
        owner: initiator.user.id,
        order: posted.order,
        issued_by: None
    };

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(rtn))
}

pub async fn handle_delete(
    initiator: Initiator,
    db: state::WebDbState,
    path: web::Path<CustomFieldPath>,
) -> error::Result<impl Responder> {
    let conn = &*db.get_conn().await?;
    security::assert::is_owner_for_custom_field(conn, &path.field_id, &initiator.user.id).await?;

    let _custom_field_entries_result = conn.execute(
        "delete from custom_field_entries where field = $1",
        &[&path.field_id]
    ).await?;

    let _custom_field_result = conn.execute(
        "delete from custom_fields where id = $1",
        &[&path.field_id]
    ).await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(None::<()>)
}