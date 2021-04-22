use actix_web::{web, http, HttpRequest, Responder};
use actix_session::{Session};
use tokio_postgres::{Client};
use serde::{Serialize, Deserialize};

use crate::error::{Result, ResponseError};
use crate::request::from;
use crate::response;
use crate::state;

#[derive(Serialize)]
pub struct MoodFieldJson {
    id: i32,
    name: String,
    minimum: Option<i32>,
    maximum: Option<i32>,
    is_range: bool,
    comment: Option<String>
}

async fn search_mood_fields(
    conn: &Client,
    owner: i32,
) -> Result<Vec<MoodFieldJson>> {
    let rows = conn.query(
        r#"
        select id, 
               name, 
               minimum, maximum, is_range, 
               comment
        from mood_fields
        where owner = $1
        order by id asc
        "#,
        &[&owner]
    ).await?;
    let mut rtn = Vec::<MoodFieldJson>::with_capacity(rows.len());

    for row in rows {
        rtn.push(MoodFieldJson {
            id: row.get(0),
            name: row.get(1),
            minimum: row.get(2),
            maximum: row.get(3),
            is_range: row.get(4),
            comment: row.get(5)
        });
    }

    Ok(rtn)
}

async fn assert_is_owner_for_mood_field(
    conn: &Client,
    field_id: i32,
    owner: i32
) -> Result<()> {
    let rows = conn.query(
        "select owner from mood_fields where id = $1",
        &[&field_id]
    ).await?;

    if rows.len() == 0 {
        return Err(ResponseError::MoodFieldNotFound(field_id));
    }

    if rows[0].get::<usize, i32>(0) != owner {
        return Err(ResponseError::PermissionDenied(
            "you don't have permission to modify this users mood field".to_owned()
        ));
    }

    Ok(())
}

pub async fn handle_get_mood_fields(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>,
) -> Result<impl Responder> {
    let accept_html = response::check_if_html_req(&req, true)?;
    let conn = &app.get_conn().await?;
    let initiator_opt = from::get_initiator(conn, session).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html())
        } else {
            Ok(response::redirect_to_path("/auth/login"))
        }
    } else if initiator_opt.is_none() {
        Err(ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();

        Ok(response::json::respond_json(
            http::StatusCode::OK,
            response::json::MessageDataJSON::build(
                "successful",
                search_mood_fields(conn, initiator.user.get_id()).await?
            )
        ))
    }
    
}

#[derive(Deserialize)]
pub struct PostMoodFieldJson {
    name: String,
    minimum: Option<i32>,
    maximum: Option<i32>,
    is_range: bool,
    comment: Option<String>
}

pub async fn handle_post_mood_fields(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    posted: web::Json<PostMoodFieldJson>,
) -> Result<impl Responder> {
    let conn = &app.get_conn().await?;

    let check = conn.query(
        "select id from mood_fields where name = $1 and owner = $2",
        &[&posted.name, &initiator.user.get_id()]
    ).await?;

    if check.len() != 0 {
        return Err(ResponseError::MoodFieldExists(posted.name.clone()));
    }

    let result = conn.query_one(
        "insert into mood_fields (name, minimum, maximum, is_range, comment, owner) values 
        ($1, $2, $3, $4, $5, $6) 
        returning id, name, minimum, maximum, is_range, comment",
        &[
            &posted.name, 
            &posted.minimum, &posted.maximum,
            &posted.is_range, 
            &posted.comment, 
            &initiator.user.get_id()
        ]
    ).await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::build(
            "successful",
            MoodFieldJson {
                id: result.get(0),
                name: result.get(1),
                minimum: result.get(2),
                maximum: result.get(3),
                is_range: result.get(4),
                comment: result.get(5)
            }
        )
    ))
}

#[derive(Deserialize)]
pub struct PutMoodFieldJson {
    name: String,
    minimum: Option<i32>,
    maximum: Option<i32>,
    is_range: bool,
    comment: Option<String>
}

#[derive(Deserialize)]
pub struct MoodFieldPath {
    field_id: i32
}

pub async fn handle_put_mood_fields_id(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    path: web::Path<MoodFieldPath>,
    posted: web::Json<PutMoodFieldJson>,
) -> Result<impl Responder> {
    let conn = &app.get_conn().await?;
    assert_is_owner_for_mood_field(conn, path.field_id, initiator.user.get_id()).await?;

    let _result = conn.query(
        r#"
        update mood_fields 
        set name = $1,
            minimum = $2,
            maximum = $3,
            is_range = $4,
            comment = $5
        where id = $6"#,
        &[
            &posted.name, 
            &posted.minimum, &posted.maximum,
            &posted.is_range, 
            &posted.comment, 
            &path.field_id
        ]
    ).await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::<Option<()>>::build(
            "successful",
            None
        )
    ))
}

pub async fn handle_delete_mood_fields_id(
    initiator: from::Initiator,
    app: web::Data<state::AppState>,
    path: web::Path<MoodFieldPath>,
) -> Result<impl Responder> {
    let conn = &app.get_conn().await?;
    assert_is_owner_for_mood_field(conn, path.field_id, initiator.user.get_id()).await?;

    let _mood_entries_result = conn.execute(
        "delete from mood_entries where field = $1",
        &[&path.field_id]
    ).await?;

    let _mood_field_result = conn.execute(
        "delete from mood_fields where id = $1",
        &[&path.field_id]
    ).await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK,
        response::json::MessageDataJSON::<Option<()>>::build(
            "successful",
            None
        )
    ))
}