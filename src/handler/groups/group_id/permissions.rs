use actix_web::HttpRequest;
use actix_web::{web, http, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::routing::path;
use crate::{db, security};
use crate::request;
use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::state;

pub async fn handle_get(
    req: HttpRequest,
    db: state::WebDbState,
    path: web::Path<path::params::GroupPath>
) -> error::Result<impl Responder> {
    let conn = &*db.pool.get().await?;
    let accept_html = response::try_check_if_html_req(&req);
    let initiator_opt = request::initiator_from_request(conn, &req).await?;

    if accept_html {
        if initiator_opt.is_some() {
            let group_id_str = path.group_id.to_string();
            let mut redirect = "/groups/".to_owned();
            redirect.push_str(&group_id_str);

            Ok(response::redirect_to_path(&redirect))
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

        let _group = match db::groups::find_id(conn, &path.group_id).await? {
            Some(g) => g,
            None => {
                return Err(error::ResponseError::GroupNotFound(path.group_id))
            }
        };

        let permissions = db::permissions::find_from_subject(
            conn,
            db::permissions::tables::GROUPS,
            &path.group_id
        ).await?;

        JsonBuilder::new(http::StatusCode::OK)
            .build(Some(permissions))
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PermissionJson {
    roll: String,
    ability: String,
    resource_table: Option<String>,
    resource_id: Option<i32>
}

pub async fn handle_put(
    initiator: request::Initiator,
    db: state::WebDbState,
    path: web::Path<path::params::GroupPath>,
    posted: web::Json<Vec<PermissionJson>>
) -> error::Result<impl Responder> {
    let conn = &mut *db.pool.get().await?;
    let posted = posted.into_inner();

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

    let mut first = true;
    let mut invalid = false;
    let rolls = db::permissions::RollDictionary::new();
    // error collection
    let mut unknown_roll: Vec<PermissionJson> = Vec::new();
    let mut invalid_ability: Vec<PermissionJson> = Vec::new();
    let mut unknown_resource_tables: Vec<PermissionJson> = Vec::new();
    let mut resource_id_not_found: Vec<PermissionJson> = Vec::new();
    let mut resource_not_allowed: Vec<PermissionJson> = Vec::new();
    // database
    let mut query = "insert into permissions (subject_table, subject_id, roll, ability, resource_table, resource_id) values".to_owned();
    let mut value_sql = String::new();
    let mut query_params = db::query::QueryParams::with_capacity(1);
    query_params.push(&path.group_id);

    for index in 0..posted.len() {
        if !invalid {
            if first {
                first = false;
            } else {
                value_sql.push(',');
            }

            value_sql.push_str("('groups',$1,");
        }

        let permission = &posted[index];
        let roll_data = match rolls.get_roll(&permission.roll) {
            Some(d) => d,
            None => {
                // roll does not exist
                unknown_roll.push(permission.clone());
                invalid = true;
                value_sql.clear();
                continue;
            }
        };

        if !invalid {
            let roll_p = query_params.push(&permission.roll).to_string();
            value_sql.push('$');
            value_sql.push_str(&roll_p);
            value_sql.push(',');
        }

        if !roll_data.check_ability(&permission.ability) {
            // invalid ability for given roll
            invalid_ability.push(permission.clone());
            invalid = true;
            value_sql.clear();
            continue;
        } else if !invalid {
            let ability_p = query_params.push(&permission.ability).to_string();
            value_sql.push('$');
            value_sql.push_str(&ability_p);
            value_sql.push(',');
        }

        if permission.resource_table.is_some() && permission.resource_id.is_some() {
            if !roll_data.allow_resource {
                // given a specific resource but not allowed for given roll
                resource_not_allowed.push(permission.clone());
                invalid = true;
                value_sql.clear();
                continue;
            }

            let resource_table = permission.resource_table.as_ref().unwrap();
            let resource_id = permission.resource_id.as_ref().unwrap();

            match resource_table.as_str() {
                db::permissions::tables::USERS => {
                    let check = conn.execute(
                        "select id from users where id = $1",
                        &[resource_id]
                    ).await?;

                    if check != 1 {
                        // resource not found
                        resource_id_not_found.push(permission.clone());
                        invalid = true;
                        value_sql.clear();
                        continue;
                    }
                },
                db::permissions::tables::GROUPS => {
                    // check to make sure that id exists
                    let check = conn.execute(
                        "select id from groups where id = $1",
                        &[resource_id]
                    ).await?;

                    if check != 1 {
                        // resource not found
                        resource_id_not_found.push(permission.clone());
                        invalid = true;
                        value_sql.clear();
                        continue;
                    }
                },
                _ => {
                    // unknown table and needs to be delt with
                    unknown_resource_tables.push(permission.clone());
                    invalid = true;
                    value_sql.clear();
                    continue;
                }
            }

            if !invalid {
                let resource_table_p = query_params.push(resource_table).to_string();
                let resource_id_p = query_params.push(resource_id).to_string();
                value_sql.push('$');
                value_sql.push_str(&resource_table_p);
                value_sql.push_str(",$");
                value_sql.push_str(&resource_id_p);
                value_sql.push(')');
            }
        } else if !invalid {
            value_sql.push_str("null,null)");
        }

        if !invalid {
            query.push_str(&value_sql);
            value_sql.clear();
        }
    }

    if invalid {
        let rtn = json!({
            "unknown_roll": unknown_roll,
            "invalid_ability": invalid_ability,
            "unknown_resource_tables": unknown_resource_tables,
            "resource_id_not_found": resource_id_not_found,
            "resource_not_allowed": resource_not_allowed,
        });

        return JsonBuilder::new(http::StatusCode::BAD_REQUEST)
            .set_error("Validation")
            .set_message("some of the permissions provided are invalid")
            .build(Some(rtn));
    }

    query.push_str("on conflict on constraint unique_permissions do nothing returning id");

    let transaction = conn.transaction().await?;

    let query_result = transaction.query(&query, query_params.slice()).await?;
    let mut returned_ids: Vec<i32> = Vec::with_capacity(query_result.len());

    for row in query_result {
        returned_ids.push(row.get(0));
    }

    transaction.execute(
        "\
        delete from permissions \
        where subject_table = 'group' and \
              subject_id = $1 and \
              id <> all($2)",
        &[&path.group_id, &returned_ids]
    ).await?;

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(None::<()>)
}