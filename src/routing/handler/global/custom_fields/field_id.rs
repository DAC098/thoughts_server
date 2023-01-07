use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;
use tokio_postgres::GenericClient;

use crate::db::tables::{
    permissions,
    custom_fields,
    global_custom_fields,
};
use crate::security::{self, InitiatorLookup, Initiator};
use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::state;

async fn get_via_id(
    conn: &impl GenericClient,
    id: &i32
) -> error::Result<global_custom_fields::GlobalCustomField> {
    if let Some(field) = global_custom_fields::find_from_id(conn, id).await? {
        Ok(field)
    } else {
        Err(error::build::global_custom_field_not_found(id))
    }
}

#[derive(Deserialize)]
pub struct FieldPath {
    field_id: i32
}

pub async fn handle_get(
    req: HttpRequest,
    security: security::state::WebSecurityState,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
    path: web::Path<FieldPath>,
) -> error::Result<impl Responder> {
    let conn = &*db.get_conn().await?;
    let accept_html = response::try_check_if_html_req(&req);
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
        .build(Some(get_via_id(conn, &path.field_id).await?))
}

#[derive(Deserialize)]
pub struct PutGlobalCustomFieldJson {
    name: String,
    comment: Option<String>,
    config: custom_fields::CustomFieldType
}

pub async fn handle_put(
    initiator: Initiator,
    db: state::WebDbState,
    posted: web::Json<PutGlobalCustomFieldJson>,
    path: web::Path<FieldPath>,
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

    let _original = get_via_id(&*conn, &path.field_id).await?;

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
        .build(Some(global_custom_fields::GlobalCustomField {
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
) -> error::Result<impl Responder> {
    let conn = &mut *db.get_conn().await?;

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

    let _original = get_via_id(conn, &path.field_id).await?;

    let transaction = conn.transaction().await?;
    transaction.execute(
        "delete from global_custom_fields where id = $1",
        &[&path.field_id]
    ).await?;

    transaction.commit().await?;
    
    JsonBuilder::new(http::StatusCode::OK)
        .build(None::<()>)
}