use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;

pub mod field_id;

use crate::db::tables::{
    permissions,
    global_custom_fields,
    custom_fields
};
use crate::security::{self, InitiatorLookup, Initiator};
use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::state;

pub async fn handle_get(
    req: HttpRequest,
    security: security::state::WebSecurityState,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
) -> error::Result<impl Responder> {
    let accept_html = response::try_check_if_html_req(&req);
    let conn = &*db.get_conn().await?;
    let lookup = InitiatorLookup::from_request(&security, conn, &req).await?;

    if accept_html {
        return if lookup.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(lookup.unwrap().user))?)
        } else {
            Ok(response::redirect_to_login(&req))
        }
    }

    let initiator = lookup.try_into()?;

    if !security::permissions::has_permission(
        &*conn, 
        &initiator.user.id, 
        permissions::rolls::GLOBAL_CUSTOM_FIELDS, 
        &[
            permissions::abilities::READ,
            permissions::abilities::READ_WRITE
        ], 
        None
    ).await? {
        return Err(error::build::permission_denied(
            "you do not have permissions to read global custom fields"
        ));
    }
    
    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(global_custom_fields::find_all(conn).await?))
}

#[derive(Deserialize)]
pub struct PostGlobalCustomFieldJson {
    name: String,
    comment: Option<String>,
    config: custom_fields::CustomFieldType
}

pub async fn handle_post(
    initiator: Initiator,
    db: state::WebDbState,
    posted: web::Json<PostGlobalCustomFieldJson>,
) -> error::Result<impl Responder> {
    let mut conn = db.get_conn().await?;
    let posted = posted.into_inner();

    if !security::permissions::has_permission(
        &*conn, 
        &initiator.user.id, 
        permissions::rolls::GLOBAL_CUSTOM_FIELDS,
        &[
            permissions::abilities::READ_WRITE
        ],
        None
    ).await? {
        return Err(error::build::permission_denied(
            "you do not have permissions to write global custom fields"
        ));
    }

    let check = conn.query(
        "select id from global_custom_fields where name = $1",
        &[&posted.name]
    ).await?;

    if check.len() != 0 {
        return Err(error::build::global_custom_field_exists(posted.name));
    }

    let config_json = serde_json::to_value(posted.config.clone())?;
    let transaction = conn.transaction().await?;
    let result = transaction.query_one(
        "\
        insert into global_custom_fields (name, comment, config) values \
        ($1, $2, $3) \
        returning id",
        &[
            &posted.name,
            &posted.comment,
            &config_json
        ]
    ).await?;

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(global_custom_fields::GlobalCustomField {
            id: result.get(0),
            name: posted.name,
            comment: posted.comment,
            config: posted.config
        }))
}