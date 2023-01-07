use std::io::{Write, Read, Seek};
use std::fs::File;
use std::path::PathBuf;

use futures_util::stream::StreamExt;
use actix_web::{web, web::Buf, http, HttpRequest, Responder};
use serde::Deserialize;

use crate::db;

pub mod audio_id;

use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::state;
use crate::security::{initiator, Initiator};
use crate::security;
use crate::util;

#[derive(Deserialize)]
pub struct EntryIdAudioPath {
    user_id: Option<i32>,
    entry_id: i32
}

pub async fn handle_get(
    req: HttpRequest,
    security: security::state::WebSecurityState,
    db: state::WebDbState,
    path: web::Path<EntryIdAudioPath>
) -> error::Result<impl Responder> {
    let path = path.into_inner();
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
    let is_private: Option<bool>;
    let owner: i32;

    if let Some(user_id) = path.user_id {
        if !security::permissions::has_permission(
            &*conn, 
            &initiator.user.id, 
            db::permissions::rolls::USERS_ENTRIES, 
            &[
                db::permissions::abilities::READ
            ],
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
            db::permissions::rolls::ENTRIES, 
            &[
                db::permissions::abilities::READ,
                db::permissions::abilities::READ_WRITE
            ], 
            None
        ).await? {
            return Err(error::build::permission_denied(
                "you do not have permission to read audio entries"
            ));
        }

        owner = initiator.user.id;
        is_private = None;
    }

    security::assert::is_owner_of_entry(&*conn, &owner, &path.entry_id).await?;

    let entry = db::audio_entries::find_from_entry(
        &*conn, 
        &path.entry_id, 
        &is_private
    ).await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(entry))
}

#[derive(Deserialize)]
pub struct PostAudioEntry {
    private: bool,
    comment: Option<String>
}

struct MultipartResult {
    audio_entry: PostAudioEntry,
    audio_file: File,
    audio_file_path: PathBuf
}

async fn handle_multipart_form(
    storage: &state::WebStorageState,
    mut body: actix_multipart::Multipart
) -> error::Result<MultipartResult> {
    let mut audio_entry: Option<PostAudioEntry> = None;
    let mut audio_file: Option<File> = None;
    let mut audio_file_path: Option<PathBuf> = None;

    while let Some(item) = body.next().await {
        let mut field = item.map_err(
            |e| error::Error::new()
                .set_name("InternalError")
                .set_message("errore when parsing multipart form")
                .set_source(e)
            )?;
        let content_type = field.content_type();

        if content_type.type_() == "application" {
            if content_type.subtype() == "json" {
                let mut string_data = String::new();

                while let Some(field_item) = field.next().await {
                    match field_item {
                        Ok(chunk) => {
                            chunk.reader().read_to_string(&mut string_data)?;
                        },
                        Err(e) => {
                            return Err(error::Error::new()
                                .set_name("InternalError")
                                .set_message("problem when reading data from request body")
                                .set_source(e));
                        }
                    }
                }

                audio_entry = Some(serde_json::from_str(string_data.as_str())?);
            } else {
                return Err(error::build::bad_request(
                    "invalid content-type given for application group. only accepts application/json"
                ));
            }
        } else if content_type.type_() == "audio" {
            if content_type.subtype() == "webm" {
                let mut file;
                let tmp_path;

                if audio_file.is_some() {
                    tmp_path = audio_file_path.unwrap();
                    file = audio_file.unwrap();
                    file.rewind()?;
                } else {
                    tmp_path = util::file::get_tmp_path(storage.get_tmp_dir_ref(), "webm");
                    file = File::create(&tmp_path)?;
                }

                while let Some(field_item) = field.next().await {
                    match field_item {
                        Ok(chunk) => {
                            file.write(&chunk)?;
                        },
                        Err(e) => {
                            return Err(error::Error::new()
                                .set_name("InternalError")
                                .set_message("problem with reading file from request")
                                .set_source(e));
                        }
                    }
                }

                audio_file = Some(file);
                audio_file_path = Some(tmp_path);
            }
        }
    }

    if audio_file.is_none() {
        return Err(error::build::bad_request(
            "missing required audio file in multipart form body."
        ));
    }

    Ok(MultipartResult {
        audio_entry: audio_entry.unwrap_or(PostAudioEntry {
            private: false,
            comment: None
        }),
        audio_file: audio_file.unwrap(),
        audio_file_path: audio_file_path.unwrap()
    })
}

struct AudioWebmResult {
    audio_file: File,
    audio_file_path: PathBuf
}

async fn handle_audio_webm(
    storage: &state::WebStorageState, 
    mut body: web::Payload
) -> error::Result<AudioWebmResult> {
    let audio_file_path = util::file::get_tmp_path(storage.get_tmp_dir_ref(), "webm");
    let mut audio_file = File::create(&audio_file_path)?;

    while let Some(item) = body.next().await {
        match item {
            Ok(chunk) => {
                audio_file.write(&chunk)?;
            },
            Err(e) => {
                return Err(error::Error::new()
                    .set_name("InternalError")
                    .set_message("problem with reading file from request")
                    .set_source(e))
            }
        }
    }

    Ok(AudioWebmResult {
        audio_file,
        audio_file_path,
    })
}

pub async fn handle_post(
    req: HttpRequest,
    initiator: Initiator,
    db: state::WebDbState,
    storage: state::WebStorageState,
    path: web::Path<EntryIdAudioPath>,
    body: web::Payload,
) -> error::Result<impl Responder> {
    let path = path.into_inner();
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
            "you do not have permission to create audio entries"
        ));
    }

    security::assert::is_owner_for_entry(&*conn, &path.entry_id, &initiator.user.id).await?;

    let audio_entry;
    let audio_file;
    let audio_file_path;

    if let Some(content_type_value) = req.headers().get("content-type") {
        if content_type_value == "multipart/form-data" {
            let results = handle_multipart_form(
                &storage, 
                actix_multipart::Multipart::new(req.headers(), body)
            ).await?;

            audio_entry = results.audio_entry;
            audio_file = results.audio_file;
            audio_file_path = results.audio_file_path
        } else if content_type_value == "audio/webm" {
            let results = handle_audio_webm(&storage, body).await?;

            audio_entry = PostAudioEntry {
                private: false,
                comment: None
            };
            audio_file = results.audio_file;
            audio_file_path = results.audio_file_path;
        } else {
            if let Ok(header_value) = content_type_value.to_str() {
                return Err(error::build::bad_request(
                    format!("invalid content-type given. expect: multipart/form-data | given: {}", header_value)
                ));
            } else {
                return Err(error::build::bad_request(
                    "header value contains invalid characters. cannot display value"
                ))
            }
        }
    } else {
        return Err(error::build::bad_request(
            "no content-type specified for request body"
        ));
    }

    let transaction = conn.transaction().await?;

    let result = transaction.query_one(
        "\
        insert into audio_entries (entry, private, comment) \
        values ($1, $2, $3) \
        returning id",
        &[&path.entry_id, &audio_entry.private, &audio_entry.comment]
    ).await?;
    let id: i32 = result.get(0);

    let new_path = storage.get_audio_file_path(&initiator.user.id, &path.entry_id, &id, "webm");
    util::file::copy_file(&audio_file, new_path)?;
    std::fs::remove_file(audio_file_path)?;

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .build(Some(db::audio_entries::AudioEntry {
            id,
            private: audio_entry.private,
            comment: audio_entry.comment,
            entry: path.entry_id
        }))
}