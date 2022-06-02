use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;

use tlib::db;

pub mod field_id;

use crate::request::{initiator_from_request, Initiator};
use crate::response;
use crate::state;
use crate::security;

use response::error::{Result, ResponseError};

#[derive(Deserialize)]
pub struct CustomFieldsPath {
    user_id: Option<i32>
}

pub async fn handle_get(
    req: HttpRequest,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
    path: web::Path<CustomFieldsPath>,
) -> Result<impl Responder> {
    let accept_html = response::try_check_if_html_req(&req);
    let conn = &*db.get_conn().await?;
    let initiator_opt = initiator_from_request(conn, &req).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(initiator_opt.unwrap().user))?)
        } else {
            Ok(response::redirect_to_path("/auth/login?jump_to=/custom_fields"))
        }
    } else if initiator_opt.is_none() {
        Err(ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();
        let owner: i32;

        if let Some(user_id) = path.user_id {
            security::assert::permission_to_read(conn, &initiator.user.id, &user_id).await?;
            owner = user_id;
        } else {
            owner = initiator.user.id;
        }

        Ok(response::json::respond_json(
            http::StatusCode::OK,
            response::json::MessageDataJSON::build(
                "successful",
                db::custom_fields::find_from_owner(conn, &owner).await?
            )
        ))
    }
    
}

#[derive(Deserialize)]
pub struct PostCustomFieldJson {
    name: String,
    config: db::custom_fields::CustomFieldType,
    comment: Option<String>,
    order: i32
}

pub async fn handle_post(
    initiator: Initiator,
    db: state::WebDbState,
    posted: web::Json<PostCustomFieldJson>,
) -> Result<impl Responder> {
    let conn = &*db.get_conn().await?;

    let check = conn.query(
        "select id from custom_fields where name = $1 and owner = $2",
        &[&posted.name, &initiator.user.id]
    ).await?;

    if check.len() != 0 {
        return Err(ResponseError::CustomFieldExists(posted.name.clone()));
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

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful",
            db::custom_fields::CustomField {
                id: result.get(0),
                name: result.get(1),
                config: serde_json::from_value(result.get(2))?,
                comment: result.get(3),
                owner: initiator.user.id,
                order: posted.order,
                issued_by: None
            }
        )
    ))
}