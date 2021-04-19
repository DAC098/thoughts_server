use actix_web::{web, http, Responder};
use serde::{Serialize};

use crate::error;
use crate::state;
use crate::response;
use crate::request::from;
use crate::time;

#[derive(Serialize)]
struct MoodEntryJson {
    id: i32,
    field: String,
    low: i32,
    high: Option<i32>,
    is_range: bool,
    comment: Option<String>,
    entry: i32,
    date: String
}

pub async fn handle_get_mood_entries(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
) -> error::Result<impl Responder> {
    let conn = &app.get_conn().await?;
    let result = conn.query(
        r#"
        select mood_entries.id as id,
               mood_fields.name as field,
               mood_entries.low as low,
               mood_entries.high as high,
               mood_fields.is_range as is_range,
               mood_entries.comment as comment,
               entries.id as entry,
               entries.created as date
        from mood_entries
        join mood_fields on mood_entries.field = mood_fields.id
        join entries on mood_entries.entry = entries.id
        where entries.owner = $1
        "#,
        &[&initiator.user.get_id()]
    ).await?;

    let mut rtn = Vec::<MoodEntryJson>::with_capacity(result.len());

    for row in result {
        rtn.push(MoodEntryJson {
            id: row.get(0),
            field: row.get(1),
            low: row.get(2),
            high: row.get(3),
            is_range: row.get(4),
            comment: row.get(5),
            entry: row.get(6),
            date: time::naive_date_to_string(row.get(7))
        })
    }

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful",
            rtn
        )
    ))
}