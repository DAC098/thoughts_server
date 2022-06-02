use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;

use tlib::db;

pub mod field_id;

use crate::request::{initiator_from_request, Initiator};
use crate::response;
use crate::state;
use crate::security;

use response::error::{Result, ResponseError};

pub async fn handle_get(
    req: HttpRequest,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
) -> Result<impl Responder> {
    let accept_html = response::try_check_if_html_req(&req);
    let conn = &*db.get_conn().await?;
    let initiator_opt = initiator_from_request(conn, &req).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(initiator_opt.unwrap().user))?)
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
                db::global_custom_fields::find_all(conn).await?
            )
        ))
    }
}

#[derive(Deserialize)]
pub struct PostGlobalCustomFieldJson {
    name: String,
    comment: Option<String>,
    config: db::custom_fields::CustomFieldType
}

pub async fn handle_post(
    initiator: Initiator,
    db: state::WebDbState,
    posted: web::Json<PostGlobalCustomFieldJson>,
) -> Result<impl Responder> {
    security::assert::is_admin(&initiator)?;

    let conn = &mut *db.get_conn().await?;
    let posted = posted.into_inner();

    let check = conn.query(
        "select id from global_custom_fields where name = $1",
        &[&posted.name]
    ).await?;

    if check.len() != 0 {
        return Err(ResponseError::GlobalCustomFieldExists(posted.name));
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

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful",
            db::global_custom_fields::GlobalCustomField {
                id: result.get(0),
                name: posted.name,
                comment: posted.comment,
                config: posted.config
            }
        )
    ))
}