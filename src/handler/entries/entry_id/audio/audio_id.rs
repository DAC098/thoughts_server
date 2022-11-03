use actix_web::{web, http, HttpRequest, Responder};
use actix_files::NamedFile;
use serde::Deserialize;

use crate::db;

use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::state;
use crate::security::{initiator, Initiator};
use crate::security;

#[derive(Deserialize)]
pub struct EntryIdAudioIdPath {
    user_id: Option<i32>,
    entry_id: i32,
    audio_id: i32,
}

#[derive(Deserialize)]
pub struct EntryIdAudioIdquery {
    json: Option<String>
}

pub async fn handle_get(
    req: HttpRequest,
    security: state::WebSecurityState,
    db: state::WebDbState,
    storage: state::WebStorageState,
    path: web::Path<EntryIdAudioIdPath>,
    query: web::Query<EntryIdAudioIdquery>,
) -> error::Result<impl Responder> {
    let path = path.into_inner();
    let query = query.into_inner();
    let conn = db.get_conn().await?;
    let accept_html = response::try_check_if_html_req(&req);
    let lookup = initiator::from_request(&security, &*conn, &req).await?;

    if accept_html {
        let redirect_to = format!("/entries/{}", path.entry_id);

        return if lookup.is_some() {
            Ok(response::redirect_to_path(redirect_to.as_str()))
        } else {
            Ok(response::redirect_to_login_with(redirect_to.as_str()))
        }
    }
    
    let initiator = lookup.try_into()?;
    let check_private: bool;
    let owner: i32;
    let return_json: bool;

    if let Some(user_id) = path.user_id {
        security::assert::permission_to_read(&*conn, &initiator.user.id, &user_id).await?;
        owner = user_id;
        check_private = true;
    } else {
        owner = initiator.user.id;
        check_private = false;
    }

    security::assert::is_owner_of_entry(&*conn, &owner, &path.entry_id).await?;

    if let Some(given) = query.json {
        return_json = given.as_str() == "1";
    } else {
        return_json = false;
    }

    if let Some(audio_entry) = db::audio_entries::find_from_id(&*conn, &path.audio_id).await? {
        if audio_entry.entry != path.entry_id {
            // respond audio entry not found
            return Err(error::build::audio_entry_not_found(&path.audio_id));
        }

        if check_private && audio_entry.private {
            // responed permission denied as audio entry is private
            return Err(error::build::permission_denied(
                "you do not have permission to access this audio entry"
            ));
        }

        if return_json {
            JsonBuilder::new(http::StatusCode::OK)
                .build(Some(audio_entry))
        } else {
            Ok(NamedFile::open(
                storage.get_audio_file_path(&owner, &path.entry_id, &audio_entry.id, "webm")
            )?.into_response(&req))
        }
    } else {
        // responed audio entry not found
        Err(error::build::audio_entry_not_found(&path.audio_id))
    }
}

#[derive(Deserialize)]
pub struct PutAudioEntry {
    private: bool,
    comment: Option<String>,
}

pub async fn handle_put(
    initiator: Initiator,
    db: state::WebDbState,
    path: web::Path<EntryIdAudioIdPath>,
    posted: web::Json<PutAudioEntry>
) -> error::Result<impl Responder> {
    let path = path.into_inner();
    let posted = posted.into_inner();
    let mut conn = db.get_conn().await?;

    if !security::permissions::has_permission(
        &*conn, 
        &initiator.user.id, 
        db::permissions::rolls::ENTRIES, 
        &[
            db::permissions::abilities::READ_WRITE
        ],
        None
    ).await? {
        return Err(error::build::permission_denied(
            "you do not have permission to update audio entries"
        ));
    }

    security::assert::is_owner_for_entry(&*conn, &path.entry_id, &initiator.user.id).await?;

    let transaction = conn.transaction().await?;
    transaction.execute(
        "\
        update audio_entries \
        set private = $2, \
            comment = $3 \
        where id = $1",
        &[&path.audio_id, &posted.private, &posted.comment]
    ).await?;

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(db::audio_entries::AudioEntry {
            id: path.audio_id,
            private: posted.private,
            comment: posted.comment,
            entry: path.entry_id
        }))
}