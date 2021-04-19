use actix_web::{web, http, Responder};
use serde::{Serialize};

use crate::state;
use crate::error;
use crate::request::from;
use crate::response;
use crate::time;

#[derive(Serialize)]
pub struct MoodEntryJson {
    id: i32,
    thought: String,
    entry: i32,
    date: String
}

pub async fn handle_get_text_entries(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
) -> error::Result<impl Responder> {
    let conn = &app.get_conn().await?;
    let result = conn.query(
        r#"
        select text_entries.id as id,
               text_entries.thought as thought,
               entries.id as entry,
               entries.created as date
        from text_entries
        join entries on text_entries.entry = entries.id
        where entry.owner = $1
        "#,
        &[&initiator.user.get_id()]
    ).await?;

    let mut rtn = Vec::<MoodEntryJson>::with_capacity(result.len());

    for row in result {
        rtn.push(MoodEntryJson {
            id: row.get(0),
            thought: row.get(1),
            entry: row.get(2),
            date: time::naive_date_to_string(row.get(3))
        });
    }

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful",
            rtn
        )
    ))
}