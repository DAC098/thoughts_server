use std::io::{Write};
use std::fs::{File};

use futures_util::stream::{StreamExt};
use actix_web::{web, http, HttpRequest, Responder};
use actix_session::{Session};
use serde::{Deserialize};
use chrono::serde::{ts_seconds};

use tlib::{db};

pub mod audio_id;

use crate::response;
use crate::state;
use crate::request::from;
use crate::security;
use crate::util;
use crate::getters;

#[derive(Deserialize)]
pub struct EntryIdAudioPath {
    user_id: Option<i32>,
    entry_id: i32
}

pub async fn handle_get(
    req: HttpRequest,
    session: Session,
    db: state::WebDbState,
    path: web::Path<EntryIdAudioPath>
) -> response::error::Result<impl Responder> {
    let path = path.into_inner();
    let conn = db.get_conn().await?;
    let accept_html = response::check_if_html_req(&req, true).unwrap();
    let initiator = from::get_initiator(&conn, &session).await?;

    if accept_html {
        let redirect_to = format!("/entries/{}", path.entry_id);

        if initiator.is_some() {
            Ok(response::redirect_to_path(redirect_to.as_str()))
        } else {
            Ok(response::redirect_to_login_with(redirect_to.as_str()))
        }
    } else if initiator.is_none() {
        Err(response::error::ResponseError::Session)
    } else {
        let initiator = initiator.unwrap();
        let is_private: Option<bool>;
        let owner: i32;

        if let Some(user_id) = path.user_id {
            security::assert::permission_to_read(&*conn, &initiator.user.id, &user_id).await?;
            owner = user_id;
            is_private = Some(false);
        } else {
            owner = initiator.user.id;
            is_private = None;
        }

        security::assert::is_owner_of_entry(&*conn, &owner, &path.entry_id).await?;

        Ok(response::json::respond_json(
            http::StatusCode::OK,
            response::json::MessageDataJSON::build(
                "successful",
                db::audio_entries::find_from_entry(&*conn, &path.entry_id, &is_private).await?
            )
        ))
    }
}

#[derive(Deserialize)]
pub struct PostAudioEntry {
    private: bool
}

#[derive(Deserialize)]
pub struct EntryIdAudioQuery {
    json: PostAudioEntry
}

pub async fn handle_post(
    initiator: from::Initiator,
    db: state::WebDbState,
    storage: state::WebStorageState,
    path: web::Path<EntryIdAudioPath>,
    query: web::Query<EntryIdAudioQuery>,
    mut body: web::Payload,
) -> response::error::Result<impl Responder> {
    let query = query.into_inner();
    let path = path.into_inner();
    let mut conn = db.get_conn().await?;

    security::assert::is_owner_for_entry(&*conn, &path.entry_id, &initiator.user.id).await?;

    let transaction = conn.transaction().await?;

    let result = transaction.query_one(
        "\
        insert into audio_entries (entry, private) \
        values ($1, $2) \
        returning id",
        &[&path.entry_id, &query.json.private]
    ).await?;
    let id: i32 = result.get(0);

    // pull contents of request body into file
    let mut file = File::create(
        storage.get_audio_file_path(&initiator.user.id, &path.entry_id, &id, "webm")
    )?;

    while let Some(item) = body.next().await {
        match item {
            Ok(chunk) => {
                file.write(&chunk)?;
            },
            Err(e) => {
                return Err(response::error::ResponseError::GeneralWithInternal(
                    format!("problem with reading file from request"),
                    format!("failed to read audio file from request. {:?}", e)
                ))
            }
        }
    }

    transaction.commit().await?;

    Ok(response::json::respond_json(
        http::StatusCode::OK, 
        response::json::MessageDataJSON::build(
            "successful",
            db::audio_entries::AudioEntry {
                id,
                private: query.json.private,
                entry: path.entry_id
            }
        )
    ))
}