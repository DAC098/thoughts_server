use std::collections::HashMap;

use actix_web::{web, http, HttpRequest, Responder};
use serde::{Serialize, Deserialize};

use crate::db;

use crate::request::Initiator;
use crate::request::initiator_from_request;
use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::state;

#[derive(Serialize, Deserialize)]
pub struct BackupDataJson {
    custom_fields: Vec<db::custom_fields::CustomField>,
    tags: Vec<db::tags::Tag>,
    entries: Vec<db::composed::ComposedEntry>
}

#[derive(Serialize, Deserialize)]
pub struct BackupJson {
    version: String,
    hash: String,
    data: BackupDataJson
}

pub async fn handle_get(
    req: HttpRequest,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
) -> error::Result<impl Responder> {
    let accept_html = response::try_check_if_html_req(&req);
    let conn = &*db.get_conn().await?;
    let initiator_opt = initiator_from_request(conn, &req).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(initiator_opt.unwrap().user))?)
        } else {
            Ok(response::redirect_to_path("/auth/login"))
        }
    } else if initiator_opt.is_none() {
        Err(error::ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();
        let data = BackupDataJson {
            custom_fields: db::custom_fields::find_from_owner(conn, &initiator.user.id).await?, 
            tags: db::tags::find_from_owner(conn, initiator.user.id).await?, 
            entries: Vec::new()
        };

        Ok(response::json::respond_json(
            http::StatusCode::OK,
            BackupJson {
                version: "1.0.0".to_owned(),
                data, 
                hash: "".to_owned()
            }
        ))
    }
}

pub async fn handle_post(
    initiator: Initiator,
    db: state::WebDbState,
    posted: web::Json<BackupJson>,
) -> error::Result<impl Responder> {
    let json_data = posted.into_inner();
    let conn = &mut *db.get_conn().await?;

    let mut custom_field_mapping: HashMap<i32, i32> = HashMap::with_capacity(json_data.data.custom_fields.len());
    let mut custom_field_config_mapping: HashMap<i32, db::custom_fields::CustomFieldType> = HashMap::with_capacity(json_data.data.custom_fields.len());
    let mut tags_mapping: HashMap<i32, i32> = HashMap::with_capacity(json_data.data.tags.len());

    let transaction = conn.transaction().await?;

    for custom_field in json_data.data.custom_fields {
        let config_json = serde_json::to_value(custom_field.config.clone()).unwrap();
        let result = transaction.query_one(
            r#"
            insert into custom_fields (name, config, "order", comment, owner) values
            ($1, $2, $3, $4, $5)
            on conflict on constraint unique_name_owner do update
            set name = excluded.name
            where custom_fields.name = excluded.name and
                  custom_fields.owner = excluded.owner
            returning id
            "#,
            &[&custom_field.name, &config_json, &custom_field.order, &custom_field.comment, &initiator.user.id]
        ).await?;

        custom_field_mapping.insert(custom_field.id, result.get(0));
        custom_field_config_mapping.insert(custom_field.id, custom_field.config);
    }

    custom_field_mapping.shrink_to_fit();
    custom_field_config_mapping.shrink_to_fit();

    for tag in json_data.data.tags {
        let result = transaction.query_one(
            r#"
            insert into tags (title, color, comment, owner) values
            ($1, $2, $3, $4)
            on conflict on constraint unique_title_owner do update
            set title = excluded.title
            where tags.title = excluded.title and
                  tags.owner = excluded.owner
            returning id
            "#,
            &[&tag.title, &tag.color, &tag.comment, &tag.owner]
        ).await?;

        tags_mapping.insert(tag.id, result.get(0));
    }

    tags_mapping.shrink_to_fit();

    for entry in json_data.data.entries {
        let result = transaction.query(
            r#"
            insert into entries (day, owner) values
            ($1, $2)
            on conflict on constraint unique_day_owner_key do nothing
            returning id
            "#,
            &[&entry.entry.day, &initiator.user.id]
        ).await?;

        if result.is_empty() {
            continue;
        }

        let entry_id: i32 = result[0].get(0);

        for tag in entry.tags {
            let tag_id_opt = tags_mapping.get(&tag);

            if let Some(tag_id) = tag_id_opt {
                let _tag_result = transaction.execute(
                    r#"insert into entries2tags (tag, entry) values ($1, $2)"#,
                    &[tag_id, &entry_id]
                ).await?;
            }
        }

        for (field_id, custom_field_entry) in entry.custom_field_entries {
            let custom_field_id_opt = custom_field_mapping.get(&field_id);

            if let Some(custom_field_id) = custom_field_id_opt {
                let config = custom_field_config_mapping.get(&field_id).unwrap();

                db::validation::verifiy_custom_field_entry(&config, &custom_field_entry.value)?;

                let value_json = serde_json::to_value(custom_field_entry.value.clone()).unwrap();
                let _custom_field_entry_result = transaction.execute(
                    r#"
                    insert into custom_field_entries (field, value, comment, entry) values
                    ($1, $2, $3, $4)
                    "#,
                    &[custom_field_id, &value_json, &custom_field_entry.comment, &entry_id]
                ).await?;
            }
        }
    }

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .set_message("okay")
        .build(None::<()>)
}