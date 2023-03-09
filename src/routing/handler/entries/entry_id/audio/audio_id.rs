//! handles working with audio entries on a singular basis

use std::str::FromStr;

use actix_web::{web, http, HttpRequest, Responder};
use actix_files::NamedFile;
use serde::Deserialize;

use crate::db::tables::{entries, audio_entries, permissions};
use crate::net::http::{error, response::{self, json::JsonBuilder}};
use crate::state;
use crate::security::{self, InitiatorLookup, Initiator};
use crate::routing;

/// retrieves a single audio entry with the given entry and audio id
///
/// GET /entries/{entry_id}/audio/{audio_id}
/// GET /users/{user_id}/entries/{entry_id}/audio/{audio_id}
pub async fn handle_get(
    req: HttpRequest,
    security: security::state::WebSecurityState,
    db: state::WebDbState,
    storage: state::WebStorageState,
    path: web::Path<routing::path::params::EntryAudioPath>,
) -> error::Result<impl Responder> {
    let path = path.into_inner();
    let conn = db.get_conn().await?;
    let accept_html = response::try_check_if_html_req(&req);
    let lookup = InitiatorLookup::from_request(&security, &*conn, &req).await?;

    if accept_html {
        let redirect_to = format!("/entries/{}", path.entry_id);

        return if lookup.is_some() {
            Ok(response::redirect_to_path(redirect_to.as_str()))
        } else {
            Ok(response::redirect_to_login_with(redirect_to.as_str()))
        }
    }

    let initiator = lookup.try_into()?;
    let owner: i32;
    let mut is_private = None::<bool>;

    if let Some(user_id) = path.user_id {
        if !security::permissions::has_permission(
            &*conn,
            &initiator.user.id,
            permissions::rolls::USERS_ENTRIES,
            &[permissions::abilities::READ],
            Some(&user_id)
        ).await? {
            return Err(error::build::permission_denied(
                "you do not have permission to read this users audio entries"
            ));
        }

        owner = user_id;
        is_private = Some(false);
    } else {
        if !security::permissions::has_permission(
            &*conn,
            &initiator.user.id,
            permissions::rolls::ENTRIES,
            &[
                permissions::abilities::READ,
                permissions::abilities::READ_WRITE,
            ],
            None
        ).await? {
            return Err(error::build::permission_denied(
                "you do not have permission to write audio entries"
            ));
        }

        owner = initiator.user.id;
    }

    let Some(_entry) = entries::from_user_and_id(&*conn, &owner, &path.entry_id).await? else {
        return Err(error::build::entry_not_found(&path.entry_id));
    };

    if let Some(audio_entry) = audio_entries::find_from_id(
        &*conn, 
        &path.audio_id,
        &is_private
    ).await? {
        let mime = {
            let known = format!("{}/{}", audio_entry.mime_type, audio_entry.mime_subtype);

            mime::Mime::from_str(&known)?
        };
        let file = NamedFile::open(storage.get_audio_file_path(&owner, &path.entry_id, &path.audio_id, "webm"))?
            .set_content_type(mime);

        Ok(file.into_response(&req))
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

/// updates an single audio id
///
/// PUT /entries/{entry_id}/audio/{audio_id}
pub async fn handle_put(
    initiator: Initiator,
    db: state::WebDbState,
    path: web::Path<routing::path::params::EntryAudioPath>,
    posted: web::Json<PutAudioEntry>
) -> error::Result<impl Responder> {
    let path = path.into_inner();
    let posted = posted.into_inner();
    let mut conn = db.get_conn().await?;

    if !security::permissions::has_permission(
        &*conn, 
        &initiator.user.id, 
        permissions::rolls::ENTRIES, 
        &[
            permissions::abilities::READ_WRITE
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
        .build(None::<()>)
}
