use std::fmt::{Write};

use actix_web::{web, http, HttpRequest, Responder};
use tokio_postgres::{Client};
use chrono::{NaiveDate, Local};
use serde::{Serialize, Deserialize};

use crate::db;
use crate::response;
use crate::{error as app_error};
use crate::state;
use crate::time;
use crate::request::from;

#[derive(Serialize)]
struct MoodEntryJson {
    id: i32,
    field: String,
    field_id: i32,
    low: i32,
    high: Option<i32>,
    is_range: bool,
    comment: Option<String>
}

#[derive(Serialize)]
struct TextEntryJson {
    id: i32,
    thought: String
}

#[derive(Serialize)]
struct EntryJson {
    id: i32,
    created: String,
    owner: i32,
    mood_entries: Vec<MoodEntryJson>,
    text_entries: Vec<TextEntryJson>
}

async fn search_text_entries(
    conn: &Client,
    owner: i32,
    entry_id_opt: Option<i32>
) -> app_error::Result<Vec<TextEntryJson>> {
    let mut query_str = r#"
    select text_entries.id as id,
           text_entries.thought as thought
    from text_entries
    join entries on
        text_entries.entry = entries.id
    where entries.id = $1"#.to_owned();
    let mut query_slice: Vec<&(dyn tokio_postgres::types::ToSql + std::marker::Sync)> = vec!(&owner);
    let entry_id = entry_id_opt.unwrap_or(0);

    if entry_id != 0 {
        write!(&mut query_str, " and entries.id = $2")?;
        query_slice.push(&entry_id);
    }

    write!(&mut query_str, "\n    order by text_entries.id asc")?;

    let rows = conn.query(query_str.as_str(), &query_slice[..]).await?;
    let mut rtn = Vec::<TextEntryJson>::with_capacity(rows.len());

    for row in rows {
        rtn.push(TextEntryJson{
            id: row.get(0),
            thought: row.get(1)
        });
    }

    Ok(rtn)
}

async fn search_mood_entries(
    conn: &Client,
    owner: i32,
    entry_id_opt: Option<i32>
) -> app_error::Result<Vec<MoodEntryJson>> {
    let mut query_str = r#"
    select mood_entries.id as id,
           mood_fields.name as field,
           mood_fields.id as field_id,
           mood_entries.low as low,
           mood_entries.high as high,
           mood_fields.is_range as is_range,
           mood_entries.comment as comment
    from mood_entries
    join mood_fields on mood_entries.field = mood_fields.id
    join entries on mood_entries.entry = entries.id
    where entries.owner = $1"#.to_owned();
    let mut query_slice: Vec<&(dyn tokio_postgres::types::ToSql + std::marker::Sync)> = vec!(&owner);
    let entry_id = entry_id_opt.unwrap_or(0);

    if entry_id != 0 {
        write!(&mut query_str, " and entries.id = $2")?;
        query_slice.push(&entry_id);
    }

    write!(&mut query_str, "\n    order by mood_entries.field asc")?;

    let rows = conn.query(query_str.as_str(), &query_slice[..]).await?;
    let mut rtn = Vec::<MoodEntryJson>::with_capacity(rows.len());

    for row in rows {
        rtn.push(MoodEntryJson {
            id: row.get(0),
            field: row.get(1),
            field_id: row.get(2),
            low: row.get(3),
            high: row.get(4),
            is_range: row.get(5),
            comment: row.get(6)
        });
    }

    Ok(rtn)
}

async fn search_entries(
    conn: &Client, 
    owner: i32, 
    entry_id_opt: Option<i32>
) -> app_error::Result<Vec<EntryJson>> {
    let mut query_str = "select id, created, owner from entries where owner = $1".to_owned();
    let mut query_slice: Vec<&(dyn tokio_postgres::types::ToSql + std::marker::Sync)> = vec!(&owner);
    let entry_id = entry_id_opt.unwrap_or(0);

    if entry_id != 0 {
        write!(&mut query_str, " and id = $2")?;
        query_slice.push(&entry_id);
    }

    write!(&mut query_str, " order by created desc")?;

    let rows = conn.query(query_str.as_str(), &query_slice[..]).await?;
    let mut rtn: Vec<EntryJson> = Vec::<EntryJson>::with_capacity(rows.len());

    for row in rows {
        let entry_id: i32 = row.get(0);

        rtn.push(EntryJson {
            id: entry_id,
            created: time::naive_date_to_string(row.get(1)),
            owner: row.get(2),
            mood_entries: search_mood_entries(conn, owner, Some(entry_id)).await?,
            text_entries: search_text_entries(conn, owner, Some(entry_id)).await?
        });
    }

    Ok(rtn)
}


fn get_created_naive(created: &String) -> app_error::Result<NaiveDate> {
    NaiveDate::parse_from_str(created.as_str(), "%Y-%m-%d").map_err(
        |_| app_error::ResponseError::Validation(
            format!("given date string is in an invalid format. make sure that the format is \"YYYY-mm-dd\" given: {}", created)
        )
    )
}

async fn assert_is_owner_for_entry(
    conn: &Client, 
    entry_id: i32, 
    initiator: i32
) -> app_error::Result<()> {
    let owner_result = conn.query(
        "select owner from entries where id = $1", 
        &[&entry_id]
    ).await?;

    if owner_result.len() == 0 {
        return Err(app_error::ResponseError::EntryNotFound(entry_id));
    }

    if owner_result[0].get::<usize, i32>(0) != initiator {
        return Err(app_error::ResponseError::PermissionDenied(
            "you don't have permission to modify this users entry".to_owned()
        ));
    }
    
    Ok(())
}

async fn assert_is_owner_for_mood_entry(conn: &Client, entry_id: i32, mood_id: i32) -> app_error::Result<()> {
    let check = conn.query(
        "select entry from mood_entries where id = $1",
        &[&mood_id]
    ).await?;

    if check.len() == 0 {
        return Err(app_error::ResponseError::MoodEntryNotFound(mood_id));
    }

    if check[0].get::<usize, i32>(0) != entry_id {
        return Err(app_error::ResponseError::PermissionDenied(
            "you don't have permission to modify this users mood entry".to_owned()
        ));
    }

    Ok(())
}

async fn assert_is_owner_for_text_entry(conn: &Client, entry_id: i32, text_id: i32) -> app_error::Result<()> {
    let check = conn.query(
        "select entry from text_entries where id = $1",
        &[&text_id]
    ).await?;

    if check.len() == 0 {
        return Err(app_error::ResponseError::TextEntryNotFound(text_id));
    }

    if check[0].get::<usize, i32>(0) != entry_id {
        return Err(app_error::ResponseError::PermissionDenied(
            "you don't have permission to modify this users text entry".to_owned()
        ));
    }

    Ok(())
}

/**
 * GET /entries
 * returns the root html if requesting html. otherwise will send back a list of
 * available and allowed entries for the current user from the session
 */
pub async fn handle_get_entries(
    req: HttpRequest, 
    initiator: from::Initiator, 
    app: web::Data<state::AppState>
) -> app_error::Result<impl Responder> {
    if let Ok(accept_html) = response::check_if_html_req(&req) {
        if accept_html {
            return Ok(response::respond_index_html());
        }
    }

    let conn = &app.get_conn().await?;
    let rtn = search_entries(conn, initiator.user.get_id(), None).await?;

    return Ok(
        response::json::respond_json(
            http::StatusCode::OK, 
            response::json::MessageDataJSON::build(
                "successful",
                rtn
            )
        )
    );
}

#[derive(Deserialize)]
pub struct PostEntryJson {
    created: Option<String>
}

#[derive(Serialize)]
pub struct PostEntryResultJson {
    id: i32,
    created: String
}

/**
 * POST /entries
 * creates a new entry when given a date for the current user from the session.
 * will also create text and mood entries if given as well
 */
pub async fn handle_post_entries(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    posted: web::Json<PostEntryJson>
) -> app_error::Result<impl Responder> {
    let conn = &app.get_conn().await?;
    let created = match &posted.created {
        Some(s) => get_created_naive(s)?,
        None => Local::today().naive_local()
    };

    let check = conn.query(
        "select id from entries where created = $1 and owner = $2",
        &[&created, &initiator.user.get_id()]
    ).await?;

    if check.len() != 0 {
        return Err(app_error::ResponseError::EntryExists(time::naive_date_to_string(created)));
    }

    let result = conn.query_one(
        "insert into entries (created, owner) values ($1, $2) returning id, created, owner",
        &[&created, &initiator.user.get_id_ref()]
    ).await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful", 
            PostEntryResultJson {
                id: result.get(0),
                created: time::naive_date_to_string(result.get(1))
            }
        )
    ))
}

#[derive(Deserialize)]
pub struct EntryPath {
    entry_id: i32
}

/**
 * GET /entries/{id}
 * returns the requested entry with additional information for the current user
 * given the session
 */
pub async fn handle_get_entries_id(
    req: HttpRequest,
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    path: web::Path<EntryPath>
) -> app_error::Result<impl Responder> {
    if let Ok(accept_html) = response::check_if_html_req(&req) {
        if accept_html {
            return Ok(response::respond_index_html());
        }
    }

    let conn = &app.get_conn().await?;
    let rtn = search_entries(conn, initiator.user.get_id(), Some(path.entry_id)).await?;

    if rtn.len() == 0 {
        return Err(app_error::ResponseError::EntryNotFound(path.entry_id));
    }

    Ok(response::json::respond_json(
        http::StatusCode::OK, 
        response::json::MessageDataJSON::build(
            "successful",
            &rtn[0]
        )
    ))
}

#[derive(Deserialize)]
pub struct PutEntryJson {
    created: String
}

/**
 * PUT /entries/{id}
 * updates the requested entry with mood or text entries for the current
 * user
 */
pub async fn handle_put_entries_id(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    path: web::Path<EntryPath>,
    posted: web::Json<PutEntryJson>
) -> app_error::Result<impl Responder> {
    let conn = &app.get_conn().await?;
    let created = get_created_naive(&posted.created)?;
    assert_is_owner_for_entry(conn, path.entry_id, initiator.user.get_id()).await?;

    let _result = conn.query(
        "update entries set created = $1 where id = $2",
        &[&created, &path.entry_id]
    ).await?;
    
    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::<Option<()>>::build(
            "successful",
            None
        )
    ))
}

/**
 * DELETE /entries/{id}
 */
pub async fn handle_delete_entries_id(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    path: web::Path<EntryPath>
) -> app_error::Result<impl Responder> {
    let conn = &app.get_conn().await?;
    assert_is_owner_for_entry(conn, path.entry_id, initiator.user.get_id()).await?;

    let _text_result = conn.execute(
        "delete from text_entries where entry = $1",
        &[&path.entry_id]
    ).await?;

    let _mood_result = conn.execute(
        "delete from mood_entries where entry = $1",
        &[&path.entry_id]
    ).await?;

    let _entry_result = conn.execute(
        "delete from entries where id = $1",
        &[&path.entry_id]
    ).await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::<Option<()>>::build(
            "successful",
            None
        )
    ))
}

/**
 * GET /entries/{entry_id}/mood_entries
 */
pub async fn handle_get_entries_id_mood_entries(
    req: HttpRequest,
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    path: web::Path<EntryPath>
) -> app_error::Result<impl Responder> {
    if let Ok(accept_html) = response::check_if_html_req(&req) {
        if accept_html {
            return Ok(response::respond_index_html());
        }
    }
    
    let conn = &app.get_conn().await?;
    assert_is_owner_for_entry(conn, path.entry_id, initiator.user.get_id()).await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK, 
        response::json::MessageDataJSON::build(
            "successful",
            search_mood_entries(conn, initiator.user.get_id(), Some(path.entry_id)).await?
        )
    ))
}

#[derive(Deserialize)]
pub struct PostMoodEntryJson {
    field_id: i32,
    low: i32,
    high: Option<i32>,
    comment: Option<String>
}

/**
 * POST /entries/{entry_id}/mood_entries
 */
pub async fn handle_post_entries_id_mood_entries(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    path: web::Path<EntryPath>,
    posted: web::Json<Vec<PostMoodEntryJson>>
) -> app_error::Result<impl Responder> {
    let conn = &app.get_conn().await?;
    assert_is_owner_for_entry(conn, path.entry_id, initiator.user.get_id()).await?;
    
    let mut rtn: Vec<MoodEntryJson> = vec!();

    for post in posted.into_inner() {
        let field = conn.query(
            "select owner, name, is_range, id from mood_fields where owner = $1 and id = $2", 
            &[&initiator.user.get_id(), &post.field_id]
        ).await?;

        if field.len() == 0 {
            return Err(app_error::ResponseError::MoodFieldNotFound(post.field_id));
        }

        if field[0].get::<usize, i32>(0) != initiator.user.get_id() {
            return Err(app_error::ResponseError::PermissionDenied(
                format!("you do not haver permission to create a mood entry using this field id: {}", field[0].get::<usize, i32>(0))
            ));
        }

        if field[0].get::<usize, bool>(2) {
            let high = match post.high {
                Some(h) => h,
                None => Err(app_error::ResponseError::Validation(
                    "the field being used requires the high value to be specified but none was given".to_owned()
                ))?
            };

            if high < post.low {
                Err(app_error::ResponseError::Validation(
                    "the high value cannot be less than the low value given".to_owned()
                ))?;
            }

            let result = conn.query_one(r#"
                insert into mood_entries (field, low, high, comment, entry) values
                ($1, $2, $3, $4, $5)
                returning id
                "#, &[&post.field_id, &post.low, &high, &post.comment, &path.entry_id]
            ).await?;

            rtn.push(MoodEntryJson {
                id: result.get(0),
                field: field[0].get(1),
                field_id: field[0].get(3),
                low: post.low,
                high: post.high,
                is_range: field[0].get(2),
                comment: post.comment
            });
        } else {
            let result = conn.query_one(r#"
                insert into mood_entries (field, low, comment, entry) values
                ($1, $2, $3, $4)
                returning id
                "#, &[&post.field_id, &post.low, &post.comment, &path.entry_id]
            ).await?;

            rtn.push(MoodEntryJson {
                id: result.get(0),
                field: field[0].get(1),
                field_id: field[0].get(3),
                low: post.low,
                high: None,
                is_range: field[0].get(2),
                comment: post.comment
            });
        }
    }
    
    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful",
            rtn
        )
    ))
}

#[derive(Deserialize)]
pub struct EntryMoodPath {
    entry_id: i32,
    mood_id: i32
}

#[derive(Deserialize)]
pub struct PutMoodEntryJson {
    low: i32,
    high: Option<i32>,
    comment: Option<String>
}

pub async fn get_mood_field_via_mood_entry(
    conn: &Client,
    mood_id: i32,
) -> app_error::Result<db::mood_fields::MoodField> {
    let result = conn.query(
        "select field from mood_entries where id = $1",
        &[&mood_id]
    ).await?;

    if result.len() == 0 {
        return Err(app_error::ResponseError::MoodEntryNotFound(mood_id));
    }

    let mood_field = db::mood_fields::MoodField::find_id(conn, result[0].get(0)).await?;

    if mood_field.is_none() {
        return Err(app_error::ResponseError::MoodFieldNotFound(result[0].get(0)));
    }

    Ok(mood_field.unwrap())
}

/**
 * PUT /entries/{entry_id}/mood_entries/{mood_id}
 */
pub async fn handle_put_entries_id_mood_entries_id(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    path: web::Path<EntryMoodPath>,
    posted: web::Json<PutMoodEntryJson>,
) -> app_error::Result<impl Responder> {
    let conn = &app.get_conn().await?;
    assert_is_owner_for_entry(conn, path.entry_id, initiator.user.get_id()).await?;
    assert_is_owner_for_mood_entry(conn, path.entry_id, path.mood_id).await?;
    let mood_field = get_mood_field_via_mood_entry(conn, path.mood_id).await?;

    if mood_field.get_is_range() {
        let high = match &posted.high {
            Some(h) => h,
            None => Err(app_error::ResponseError::Validation(
                "the field being used requires the high value to be specified but none was given".to_owned()
            ))?
        };

        if high < &posted.low {
            Err(app_error::ResponseError::Validation(
                "the high value cannot be less than the low value given".to_owned()
            ))?;
        }

        let _result = conn.execute(r#"
            update mood_entries
            set low = $1,
                high = $2,
                comment = $3
            where id = $4
            "#, &[&posted.low, high, &posted.comment, &path.mood_id]
        ).await?;

        Ok(response::json::respond_json(
            http::StatusCode::OK,
            response::json::MessageDataJSON::<Option<()>>::build(
                "successful",
                None
            )
        ))
    } else {
        let _result = conn.execute(r#"
            update mood_entries
            set low = $1,
                comment = $2
            where id = $3
            "#, &[&posted.low, &posted.comment, &path.mood_id]
        ).await?;

        Ok(response::json::respond_json(
            http::StatusCode::OK,
            response::json::MessageDataJSON::<Option<()>>::build(
                "successful",
                None
            )
        ))
    }
}

/**
 * DELETE /entries/{entry_id}/mood_entries/{mood_id}
 */
pub async fn handle_delete_entries_id_mood_entries_id(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    path: web::Path<EntryMoodPath>,
) -> app_error::Result<impl Responder> {
    let conn = &app.get_conn().await?;
    assert_is_owner_for_entry(conn, path.entry_id, initiator.user.get_id()).await?;
    assert_is_owner_for_mood_entry(conn, path.entry_id, path.mood_id).await?;

    let _result = conn.execute(
        "delete from mood_entries where id = $1 and entry = $2",
        &[&path.mood_id, &path.entry_id]
    ).await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::<Option<()>>::build(
            "successful",
            None
        )
    ))
}

/**
 * GET /entries/{entry_id}/text_entries
 */
pub async fn handle_get_entries_id_text_entries(
    req: HttpRequest,
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    path: web::Path<EntryPath>,
) -> app_error::Result<impl Responder> {
    if let Ok(accept_html) = response::check_if_html_req(&req) {
        if accept_html {
            return Ok(response::respond_index_html());
        }
    }

    let conn = &app.get_conn().await?;
    assert_is_owner_for_entry(conn, path.entry_id, initiator.user.get_id()).await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful",
            search_text_entries(conn, initiator.user.get_id(), Some(path.entry_id)).await?
        )
    ))
}

#[derive(Deserialize)]
pub struct PostTextEntryJson {
    thought: String
}

/**
 * POST /entries/{entry_id}/text_entries
 */
pub async fn handle_post_entries_id_text_entries(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    path: web::Path<EntryPath>,
    posted: web::Json<Vec<PostTextEntryJson>>,
) -> app_error::Result<impl Responder> {
    let conn = &app.get_conn().await?;
    assert_is_owner_for_entry(conn, path.entry_id, initiator.user.get_id()).await?;

    let mut rtn: Vec<TextEntryJson> = vec!();

    for post in posted.into_inner() {
        let result = conn.query_one(
            r#"
            insert into text_entries (thought, entry) values 
            ($1, $2)
            returning id,
                      thought
            "#,
            &[&post.thought, &path.entry_id]
        ).await?;

        rtn.push(TextEntryJson {
            id: result.get(0),
            thought: result.get(1)
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

#[derive(Deserialize)]
pub struct TextEntryPath {
    entry_id: i32,
    text_id: i32
}

#[derive(Deserialize)]
pub struct PutTextEntryJson {
    thought: String
}

/**
 * PUT /entries/{entry_id}/text_entries/{text_id}
 */
pub async fn handle_put_entries_id_text_entries_id(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    path: web::Path<TextEntryPath>,
    posted: web::Json<PutTextEntryJson>
) -> app_error::Result<impl Responder> {
    let conn = &app.get_conn().await?;
    assert_is_owner_for_entry(conn, path.entry_id, initiator.user.get_id()).await?;
    assert_is_owner_for_text_entry(conn, path.entry_id, path.text_id).await?;

    let _result = conn.execute(
        "update text_entries set thought = $1 where id = $2 and entry = $3",
        &[&posted.thought, &path.text_id, &path.entry_id]
    ).await?;
    
    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::<Option<()>>::build(
            "successful",
            None
        )
    ))
}

/**
 * DELETE /entries/{entry_id}/text_entries/{text_id}
 */
pub async fn handle_delete_entries_id_text_entries_id(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    path: web::Path<TextEntryPath>,
) -> app_error::Result<impl Responder> {
    let conn = &app.get_conn().await?;
    assert_is_owner_for_entry(conn, path.entry_id, initiator.user.get_id()).await?;
    assert_is_owner_for_text_entry(conn, path.entry_id, path.text_id).await?;

    let _result = conn.execute(
        "delete from text_entries where id = $1",
        &[&path.text_id]
    ).await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::<Option<()>>::build(
            "successful",
            None
        )
    ))
}