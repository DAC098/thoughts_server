use actix_web::{web, http, HttpRequest, Responder};
use actix_session::{Session};
use serde::{Deserialize};

use tlib::{db};

use crate::request::from;
use crate::response;
use crate::state;
use crate::security;

use response::error::{Result, ResponseError};

#[derive(Deserialize)]
pub struct CustomFieldPath {
    user_id: Option<i32>,
    field_id: i32
}

pub async fn handle_get(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>,
    path: web::Path<CustomFieldPath>
) -> Result<impl Responder> {
    let accept_html = response::check_if_html_req(&req, true)?;
    let conn = &*app.get_conn().await?;
    let initiator_opt = from::get_initiator(conn, &session).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(Some(initiator_opt.unwrap().user)))
        } else {
            let redirect = format!("/auth/login?jump_to=/custom_fields/{}", path.field_id);
            Ok(response::redirect_to_path(redirect.as_str()))
        }
    } else if initiator_opt.is_none() {
        Err(ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();
        let owner: i32;

        if let Some(user_id) = path.user_id {
            security::assert::permission_to_read(conn, initiator.user.id, user_id).await?;
            owner = user_id;
        } else {
            owner = initiator.user.id;
        }

        if let Some(field) = db::custom_fields::find_from_id(conn, &path.field_id).await? {
            if field.owner == owner {
                Ok(response::json::respond_json(
                    http::StatusCode::OK,
                    response::json::MessageDataJSON::build(
                        "successful",
                        field
                    )
                ))
            } else {
                Err(ResponseError::PermissionDenied(
                    format!("custom field owner mis-match. requested custom field is not owned by {}", owner)
                ))
            }
        } else {
            Err(ResponseError::CustomFieldNotFound(path.field_id))
        }
    }
}

#[derive(Deserialize)]
pub struct PutCustomFieldJson {
    name: String,
    config: db::custom_fields::CustomFieldType,
    comment: Option<String>,
    order: i32
}

pub async fn handle_put(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    path: web::Path<CustomFieldPath>,
    posted: web::Json<PutCustomFieldJson>,
) -> Result<impl Responder> {
    let conn = &*app.get_conn().await?;
    security::assert::is_owner_for_custom_field(conn, path.field_id, initiator.user.id).await?;

    let config_json = serde_json::to_value(posted.config.clone())?;
    let result = conn.query_one(
        r#"
        update custom_fields
        set name = $1,
            config = $2,
            comment = $3,
            "order" = $4
        where id = $5
        returning name, comment
        "#,
        &[
            &posted.name,
            &config_json,
            &posted.comment,
            &posted.order,
            &path.field_id
        ]
    ).await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful",
            db::custom_fields::CustomField {
                id: path.field_id,
                name: result.get(0),
                config: posted.config.clone(),
                comment: result.get(1),
                owner: initiator.user.id,
                order: posted.order,
                issued_by: None
            }
        )
    ))
}

pub async fn handle_delete(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    path: web::Path<CustomFieldPath>,
) -> Result<impl Responder> {
    let conn = &*app.get_conn().await?;
    security::assert::is_owner_for_custom_field(conn, path.field_id, initiator.user.id).await?;

    let _custom_field_entries_result = conn.execute(
        "delete from custom_field_entries where field = $1",
        &[&path.field_id]
    ).await?;

    let _custom_field_result = conn.execute(
        "delete from custom_fields where id = $1",
        &[&path.field_id]
    ).await?;

    Ok(response::json::respond_okay())
}