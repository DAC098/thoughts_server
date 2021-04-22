use std::fmt::{Write};
use std::convert::{Into};

use actix_web::{web, http, HttpRequest, Responder};
use actix_session::{Session};
use tokio_postgres::{Client, GenericClient};
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
    where entries.owner = $1"#.to_owned();
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
    let mut query_str = "select id, day, owner from entries where owner = $1".to_owned();
    let mut query_slice: Vec<&(dyn tokio_postgres::types::ToSql + std::marker::Sync)> = vec!(&owner);
    let entry_id = entry_id_opt.unwrap_or(0);

    if entry_id != 0 {
        write!(&mut query_str, " and id = $2")?;
        query_slice.push(&entry_id);
    }

    write!(&mut query_str, " order by day desc")?;

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

async fn get_mood_field_via_id(
    conn: &impl GenericClient,
    initiator: i32,
    field_id: i32,
) -> app_error::Result<db::mood_fields::MoodField> {
    let field = match db::mood_fields::find_id(conn, field_id).await? {
        Some(field) => field,
        None => Err(app_error::ResponseError::MoodFieldNotFound(field_id))?
    };

    if field.get_owner() != initiator {
        return Err(app_error::ResponseError::PermissionDenied(
            format!("you do not haver permission to create a mood entry using this field id: {}", field.get_owner())
        ));
    }

    Ok(field)
}

async fn get_mood_field_via_mood_entry(
    conn: &impl GenericClient,
    initiator: i32,
    mood_id: i32,
) -> app_error::Result<db::mood_fields::MoodField> {
    let result = conn.query(
        r#"
        select mood_entries.field,
               entries.owner 
        from mood_entries 
        join entries on mood_entries.entry = entries.id
        where mood_entries.id = $1
        "#,
        &[&mood_id]
    ).await?;

    if result.len() == 0 {
        return Err(app_error::ResponseError::MoodEntryNotFound(mood_id));
    }

    if result[0].get::<usize,i32>(1) != initiator {
        return Err(app_error::ResponseError::PermissionDenied(
            format!("you do not own this mood entry. mood entry: {}", mood_id)
        ));
    }

    get_mood_field_via_id(conn, initiator, result[0].get(0)).await
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

/**
 * GET /entries
 * returns the root html if requesting html. otherwise will send back a list of
 * available and allowed entries for the current user from the session
 */
pub async fn handle_get_entries(
    req: HttpRequest, 
    session: Session,
    app: web::Data<state::AppState>
) -> app_error::Result<impl Responder> {
    let conn = &app.get_conn().await?;
    let accept_html = response::check_if_html_req(&req, true).unwrap();
    let initiator_opt = from::get_initiator(conn, session).await?;

    if accept_html && initiator_opt.is_some() {
        Ok(response::respond_index_html())
    } else if accept_html && initiator_opt.is_none() {
        Ok(response::redirect_to_path("/auth/login"))
    } else if initiator_opt.is_none() {
        Err(app_error::ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();

        Ok(response::json::respond_json(
            http::StatusCode::OK, 
            response::json::MessageDataJSON::build(
                "successful",
                search_entries(conn, initiator.user.get_id(), None).await?
            )
        ))
    }
}

#[derive(Deserialize)]
pub struct PostTextEntryJson {
    thought: String
}

#[derive(Deserialize)]
pub struct PostMoodEntryJson {
    field_id: i32,
    low: i32,
    high: Option<i32>,
    comment: Option<String>
}

#[derive(Deserialize)]
pub struct PostEntryJson {
    created: Option<String>,
    mood_entries: Option<Vec<PostMoodEntryJson>>,
    text_entries: Option<Vec<PostTextEntryJson>>
}

#[derive(Serialize)]
pub struct PostEntryResultJson {
    id: i32,
    created: String,
    mood_entries: Vec<MoodEntryJson>,
    text_entries: Vec<TextEntryJson>
}

#[derive(Serialize)]
pub struct PutEntryResultJson {
    id: i32,
    created: String,
    mood_entries: Option<Vec<MoodEntryJson>>,
    text_entries: Option<Vec<TextEntryJson>>
}

fn validate_range(
    low: i32,
    high: i32,
    field: &db::mood_fields::MoodField
) -> app_error::Result<()> {
    if let Some(min) = field.get_minimum() {
        if low < min {
            return Err(app_error::ResponseError::Validation(
                format!("this field has a specified minimum and the low is less than it. field {} minimum {} value {}", field.get_id(), min, low)
            ));
        }
    }

    if high < low {
        return Err(app_error::ResponseError::Validation(
            format!("the high value cannot be less than the low value given. low {} high {}", low, high)
        ));
    }

    if let Some(max) = field.get_maximum() {
        if high > max {
            return Err(app_error::ResponseError::Validation(
                format!("this field has a specified maximum and the high is greater than it. field {} maximum {} value {}", field.get_id(), max, high)
            ));
        }
    }

    Ok(())
}

fn validate_low_value(
    low: i32,
    field: &db::mood_fields::MoodField
) -> app_error::Result<()> {
    if let Some(min) = field.get_minimum() {
        if low < min {
            return Err(app_error::ResponseError::Validation(
                format!("this field has a specified minimum and the value is less than it. field {} minimum {} value {}", field.get_id(), min, low)
            ));
        }
    }

    if let Some(max) = field.get_maximum() {
        if low > max {
            return Err(app_error::ResponseError::Validation(
                format!("this field has a specified maximum and the value is greater than it. field {} maximum {} value {}", field.get_id(), max, low)
            ));
        }
    }

    Ok(())
}

fn get_posted_high(
    posted_high: Option<i32>
) -> app_error::Result<i32> {
    match posted_high {
        Some(h) => Ok(h),
        None => Err(app_error::ResponseError::Validation(
            "the field being used requires the high value to be specified but none was given".to_owned()
        ))
    }
}

struct CreateMoodEntryInfo {
    field_id: Option<i32>,
    low: i32,
    high: Option<i32>,
    comment: Option<String>
}

impl Into<CreateMoodEntryInfo> for &PostMoodEntryJson {
    fn into(self) -> CreateMoodEntryInfo {
        CreateMoodEntryInfo {
            field_id: Some(self.field_id),
            low: self.low,
            high: self.high,
            comment: match &self.comment {
                Some(c) => Some(c.clone()),
                None => None
            }
        }
    }
}

impl Into<CreateMoodEntryInfo> for &PutMoodEntryJson {
    fn into(self) -> CreateMoodEntryInfo {
        CreateMoodEntryInfo {
            field_id: self.field_id,
            low: self.low,
            high: self.high,
            comment: match &self.comment {
                Some(c) => Some(c.clone()),
                None => None
            }
        }
    }
}

async fn create_mood_entry<T: Into<CreateMoodEntryInfo>>(
    conn: &impl GenericClient,
    initiator: i32,
    entry_id: i32,
    posted: T
) -> app_error::Result<MoodEntryJson> {
    let post = posted.into();
    let field_id = match post.field_id {
        Some(id) => id,
        None => Err(app_error::ResponseError::Validation(
            "no field was specified for the new mood entry".to_owned()
        ))?
    };

    let field = get_mood_field_via_id(conn, initiator, field_id).await?;

    if field.get_owner() != initiator {
        return Err(app_error::ResponseError::PermissionDenied(
            format!("you do not haver permission to create a mood entry using this field id: {}", field.get_owner())
        ));
    }

    if field.get_is_range() {
        let high = get_posted_high(post.high)?;

        validate_range(post.low, high, &field)?;

        let result = conn.query_one(r#"
            insert into mood_entries (field, low, high, comment, entry) values
            ($1, $2, $3, $4, $5)
            returning id
            "#, &[&post.field_id, &post.low, &high, &post.comment, &entry_id]
        ).await?;

        Ok(MoodEntryJson {
            id: result.get(0),
            field: field.get_name(),
            field_id: field.get_id(),
            low: post.low,
            high: post.high,
            is_range: field.get_is_range(),
            comment: post.comment.clone()
        })
    } else {
        validate_low_value(post.low, &field)?;

        let result = conn.query_one(r#"
            insert into mood_entries (field, low, comment, entry) values
            ($1, $2, $3, $4)
            returning id
            "#, &[&field_id, &post.low, &post.comment, &entry_id]
        ).await?;

        Ok(MoodEntryJson {
            id: result.get(0),
            field: field.get_name(),
            field_id: field.get_id(),
            low: post.low,
            high: None,
            is_range: field.get_is_range(),
            comment: post.comment.clone()
        })
    }
}

async fn create_mood_entries(
    conn: &impl GenericClient,
    initiator: i32,
    entry_id: i32, 
    list: &Vec<PostMoodEntryJson>
) -> app_error::Result<Vec<MoodEntryJson>> {
    let mut rtn = Vec::<MoodEntryJson>::with_capacity(list.len());

    for post in list {
        rtn.push(create_mood_entry(conn, initiator, entry_id, post).await?);
    }

    Ok(rtn)
}

#[derive(Deserialize)]
pub struct PutTextEntryJson {
    id: Option<i32>,
    thought: String
}

#[derive(Deserialize)]
pub struct PutMoodEntryJson {
    id: Option<i32>,
    field_id: Option<i32>,
    low: i32,
    high: Option<i32>,
    comment: Option<String>
}

#[derive(Deserialize)]
pub struct PutEntryJson {
    created: String,
    mood_entries: Option<Vec<PutMoodEntryJson>>,
    text_entries: Option<Vec<PutTextEntryJson>>
}

struct UpdateMoodEntryInfo {
    id: Option<i32>,
    low: i32,
    high: Option<i32>,
    comment: Option<String>
}

impl Into<UpdateMoodEntryInfo> for &PutMoodEntryJson {
    fn into(self) -> UpdateMoodEntryInfo {
        UpdateMoodEntryInfo {
            id: self.id,
            low: self.low,
            high: self.high,
            comment: match &self.comment {
                Some(c) => Some(c.clone()),
                None => None
            }
        }
    }
}

async fn update_mood_entry<T: Into<UpdateMoodEntryInfo>>(
    conn: &impl GenericClient,
    initiator: i32,
    _entry_id: i32,
    posted: T
) -> app_error::Result<MoodEntryJson> {
    let post = posted.into();
    let mood_id = match post.id {
        Some(i) => i,
        None => Err(app_error::ResponseError::Validation(
            "no mood entry id was specified to update".to_owned()
        ))?
    };
    let field = get_mood_field_via_mood_entry(conn, initiator, mood_id).await?;

    if field.get_is_range() {
        let high = get_posted_high(post.high)?;

        validate_range(post.low, high, &field)?;

        let result = conn.query_one(r#"
            update mood_entries
            set low = $1,
                high = $2,
                comment = $3
            where id = $4
            returning low, high, comment
            "#, &[&post.low, &high, &post.comment, &mood_id]
        ).await?;

        Ok(MoodEntryJson {
            id: mood_id,
            field: field.get_name(),
            field_id: field.get_id(),
            low: result.get(0),
            high: result.get(1),
            is_range: field.get_is_range(),
            comment: result.get(2)
        })
    } else {
        validate_low_value(post.low, &field)?;

        let result = conn.query_one(r#"
            update mood_entries
            set low = $1,
                comment = $2
            where id = $3
            returning low, comment
            "#, &[&post.low, &post.comment, &mood_id]
        ).await?;

        Ok(MoodEntryJson {
            id: mood_id,
            field: field.get_name(),
            field_id: field.get_id(),
            low: result.get(0),
            high: None,
            is_range: field.get_is_range(),
            comment: result.get(1)
        })
    }
}

async fn update_mood_entries(
    conn: &impl GenericClient,
    initiator: i32,
    entry_id: i32,
    posted: &Vec<PutMoodEntryJson>
) -> app_error::Result<Vec<MoodEntryJson>> {
    let mut rtn = Vec::<MoodEntryJson>::with_capacity(posted.len());

    for post in posted {
        rtn.push(
            if post.id.is_some() { update_mood_entry(conn, initiator, entry_id, post).await? }
            else { create_mood_entry(conn, initiator, entry_id, post).await? }
        );
    }

    Ok(rtn)
}

async fn delete_entries(
    conn: &impl GenericClient,
    initiator: i32,
    entry_ids: Vec<i32>,
) -> app_error::Result<()> {
    let check = conn.query(
        "select id, owner from entries where id = any($1)",
        &[&entry_ids]
    ).await?;
    let mut invalid_entries: Vec<i32> = vec!();

    for row in check {
        if row.get::<usize, i32>(1) != initiator {
            invalid_entries.push(row.get(0));
        }

        if invalid_entries.len() > 0 {
            return Err(app_error::ResponseError::PermissionDenied(
                format!("you are not allowed to delete entries owned by another user. entries ({:?})", invalid_entries)
            ));
        }
    }
    
    let _text_result = conn.execute(
        "delete from text_entries where entry = any($1)",
        &[&entry_ids]
    ).await?;

    let _mood_result = conn.execute(
        "delete from mood_entries where entry = any($1)",
        &[&entry_ids]
    ).await?;

    let _entry_result = conn.execute(
        "delete from entries where id = any($1)",
        &[&entry_ids]
    ).await?;

    Ok(())
}

pub async fn handle_get_root(
) -> app_error::Result<impl Responder> {
    Ok(response::respond_index_html())
}

/**
 * POST /entries
 * creates a new entry when given a date for the current user from the session.
 * will also create text and mood entries if given as well
 */
pub async fn handle_post_entries(
    initiator: from::Initiator,
    app_data: web::Data<state::AppState>,
    posted: web::Json<PostEntryJson>
) -> app_error::Result<impl Responder> {
    let app = app_data.into_inner();
    let mut conn = app.get_conn().await?;
    let created = match &posted.created {
        Some(s) => get_created_naive(s)?,
        None => Local::today().naive_local()
    };

    let entry_check = conn.query(
        "select id from entries where day = $1 and owner = $2",
        &[&created, &initiator.user.get_id()]
    ).await?;

    if entry_check.len() != 0 {
        return Err(app_error::ResponseError::EntryExists(time::naive_date_to_string(created)));
    }

    let transaction = conn.transaction().await?;
    let result = transaction.query_one(
        "insert into entries (day, owner) values ($1, $2) returning id, day, owner",
        &[&created, &initiator.user.get_id_ref()]
    ).await?;
    let entry_id: i32 = result.get(0);

    let mood_entries: Vec<MoodEntryJson> = match &posted.mood_entries {
        Some(m) => create_mood_entries(&transaction, initiator.user.get_id(), entry_id, m).await?,
        None => vec!()
    };

    let mut text_entries: Vec<TextEntryJson> = vec!();

    if let Some(t) = &posted.text_entries {
        for text_entry in t {
            let result = transaction.query_one(
                "insert into text_entries (thought, entry) values ($1, $2) returning id, thought",
                &[&text_entry.thought, &entry_id]
            ).await?;

            text_entries.push(TextEntryJson {
                id: result.get(0),
                thought: result.get(1)
            });
        }
    }

    transaction.commit().await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful", 
            PostEntryResultJson {
                id: result.get(0),
                created: time::naive_date_to_string(result.get(1)),
                mood_entries,
                text_entries
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
    session: Session,
    app: web::Data<state::AppState>,
    path: web::Path<EntryPath>
) -> app_error::Result<impl Responder> {
    let conn = &app.get_conn().await?;
    let accept_html = response::check_if_html_req(&req, true).unwrap();
    let initiator_opt = from::get_initiator(conn, session).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html())
        } else {
            Ok(response::redirect_to_path("/auth/login"))
        }
    } else if initiator_opt.is_none() {
        Err(app_error::ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();
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
}

/**
 * PUT /entries/{id}
 * updates the requested entry with mood or text entries for the current
 * user
 */
pub async fn handle_put_entries_id(
    initiator: from::Initiator,
    app_data: web::Data<state::AppState>,
    path: web::Path<EntryPath>,
    posted: web::Json<PutEntryJson>
) -> app_error::Result<impl Responder> {
    let app = app_data.into_inner();
    let mut conn = app.get_conn().await?;
    let created = get_created_naive(&posted.created)?;
    assert_is_owner_for_entry(&conn, path.entry_id, initiator.user.get_id()).await?;

    let transaction = conn.transaction().await?;
    let result = transaction.query_one(
        "update entries set day = $1 where id = $2 returning day",
        &[&created, &path.entry_id]
    ).await?;
    let mut rtn = PutEntryResultJson {
        id: path.entry_id,
        created: time::naive_date_to_string(result.get(0)),
        mood_entries: None,
        text_entries: None
    };

    if let Some(m) = &posted.mood_entries {
        let mood_entries = update_mood_entries(&transaction, initiator.user.get_id(), path.entry_id, m).await?;
        let mut ids = Vec::<i32>::with_capacity(mood_entries.len());

        for ent in &mood_entries {
            ids.push(ent.id);
        }

        rtn.mood_entries = Some(mood_entries);

        let left_over = transaction.query(
            "select id from mood_entries where entry = $1 and not (id = any($2))",
            &[&path.entry_id, &ids]
        ).await?;

        if left_over.len() > 0 {
            let mut to_delete = Vec::<i32>::with_capacity(left_over.len());

            for row in left_over {
                to_delete.push(row.get(0));
            }

            let _result = transaction.execute(
                "delete from mood_entries where id = any($1)",
                &[&to_delete]
            ).await?;
        }
    }

    if let Some(t) = &posted.text_entries {
        let mut ids: Vec<i32> = vec!();
        let mut text_entries: Vec<TextEntryJson> = vec!();

        for text_entry in t {
            if let Some(id) = text_entry.id {
                let check = transaction.query(
                    r#"
                    select entries.owner
                    from text_entries
                    join entries on text_entries.entry = entries.id
                    where text_entries.id = $1
                    "#,
                    &[&id]
                ).await?;

                if check.len() == 0 {
                    return Err(app_error::ResponseError::TextEntryNotFound(id));
                }

                if check[0].get::<usize, i32>(0) != initiator.user.get_id() {
                    return Err(app_error::ResponseError::PermissionDenied(
                        format!("you do not have permission to modify another users text entry. text entry: {}", id)
                    ));
                }

                let result = transaction.query_one(
                    "update text_entries set thought = $1 where id = $2 returning id, thought",
                    &[&text_entry.thought, &id]
                ).await?;

                ids.push(id);
                text_entries.push(TextEntryJson {
                    id: result.get(0),
                    thought: result.get(1)
                });
            } else {
                let result = transaction.query_one(
                    "insert into text_entries (thought, entry) values ($1, $2) returning id, thought",
                    &[&text_entry.thought, &path.entry_id]
                ).await?;

                ids.push(result.get(0));
                text_entries.push(TextEntryJson {
                    id: result.get(0),
                    thought: result.get(1)
                })
            }
        }

        rtn.text_entries = Some(text_entries);

        let left_over = transaction.query(
            "select id from text_entries where entry = $1 and not (id = any($2))",
            &[&path.entry_id, &ids]
        ).await?;

        if left_over.len() > 0 {
            let mut to_delete = Vec::<i32>::with_capacity(left_over.len());

            for row in left_over {
                to_delete.push(row.get(0));
            }

            let _result = transaction.execute(
                "delete from text_entries where id = any($1)",
                &[&to_delete]
            ).await?;
        }
    }

    transaction.commit().await?;
    
    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful",
            rtn
        )
    ))
}

/**
 * DELETE /entries/{id}
 */
pub async fn handle_delete_entries_id(
    initiator: from::Initiator,
    app_data: web::Data<state::AppState>,
    path: web::Path<EntryPath>
) -> app_error::Result<impl Responder> {
    let app = app_data.into_inner();
    let mut conn = app.get_conn().await?;
    let transaction = conn.transaction().await?;

    delete_entries(&transaction, initiator.user.get_id(), vec!(path.entry_id)).await?;

    transaction.commit().await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::<Option<()>>::build(
            "successful",
            None
        )
    ))
}