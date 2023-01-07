use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;

pub mod field_id;

use crate::db::tables::custom_fields;
use crate::security::{InitiatorLookup, Initiator};
use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::state;
use crate::security;

#[derive(Deserialize)]
pub struct CustomFieldsPath {
    user_id: Option<i32>
}

pub async fn handle_get(
    req: HttpRequest,
    security: security::state::WebSecurityState,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
    path: web::Path<CustomFieldsPath>,
) -> error::Result<impl Responder> {
    let accept_html = response::try_check_if_html_req(&req);
    let conn = &*db.get_conn().await?;
    let lookup = InitiatorLookup::from_request(&security, conn, &req).await?;

    if accept_html {
        return if lookup.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(lookup.unwrap().user))?)
        } else {
            Ok(response::redirect_to_path("/auth/login?jump_to=/custom_fields"))
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

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(custom_fields::find_from_owner(conn, &owner).await?))
}

#[derive(Deserialize, Debug)]
pub struct PostCustomFieldJson {
    name: String,
    config: custom_fields::CustomFieldType,
    comment: Option<String>,
    order: i32
}

pub async fn handle_post(
    initiator: Initiator,
    db: state::WebDbState,
    posted: web::Json<PostCustomFieldJson>,
) -> error::Result<impl Responder> {
    let conn = &*db.get_conn().await?;
    let posted = posted.into_inner();

    let check = conn.query(
        "select id from custom_fields where name = $1 and owner = $2",
        &[&posted.name, &initiator.user.id]
    ).await?;

    if check.len() != 0 {
        return Err(error::build::custom_field_exists(posted.name));
    }

    let config_json = serde_json::to_value(posted.config.clone())?;
    let result = conn.query_one(
        "\
        insert into custom_fields (name, config, comment, owner, \"order\") values \
        ($1, $2, $3, $4, $5) \
        returning id, name, config, comment",
        &[
            &posted.name, 
            &config_json,
            &posted.comment, 
            &initiator.user.id,
            &posted.order
        ]
    ).await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(custom_fields::CustomField {
            id: result.get(0),
            name: result.get(1),
            config: serde_json::from_value(result.get(2))?,
            comment: result.get(3),
            owner: initiator.user.id,
            order: posted.order,
            issued_by: None
        }))
}