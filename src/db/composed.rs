use std::collections::HashMap;

use tokio_postgres::GenericClient;
use serde::{Serialize, Deserialize};

use crate::db::{
    error, 
    tables::{
        users,
        user_data,
        user_access,
        entries,
        entry_markers,
        custom_field_entries,
        text_entries,
        entries2tags,
        entry_comments,
}
};

#[derive(Serialize, Deserialize)]
pub struct ComposedEntry {
    pub entry: entries::Entry,
    pub tags: Vec<i32>,
    pub markers: Vec<entry_markers::EntryMarker>,
    pub custom_field_entries: HashMap<i32, custom_field_entries::CustomFieldEntry>,
    pub text_entries: Vec<text_entries::TextEntry>
}

impl ComposedEntry {

    pub async fn find_from_entry(
        conn: &impl GenericClient,
        entry: &i32,
        is_private: &Option<bool>,
    ) -> error::Result<Option<ComposedEntry>> {
        if let Some(entry_rec) = entries::find_from_id(conn, entry).await? {
            let results = custom_field_entries::find_from_entry(conn, entry).await?;
            let mut custom_field_map: HashMap<i32, custom_field_entries::CustomFieldEntry> = HashMap::with_capacity(results.len());
    
            for record in results {
                custom_field_map.insert(record.field, record);
            }
    
            Ok(Some(ComposedEntry {
                entry: entry_rec,
                tags: entries2tags::find_id_from_entry(conn, entry).await?,
                markers: entry_markers::find_from_entry(conn, entry).await?,
                custom_field_entries: custom_field_map,
                text_entries: text_entries::find_from_entry(conn, entry, is_private).await?,
            }))
        } else {
            Ok(None)
        }
    }

}

#[derive(Serialize, Deserialize)]
pub struct ComposedEntryComment {
    pub user: users::UserBare,
    pub comment: entry_comments::EntryComment,
}

impl ComposedEntryComment {

    pub async fn find_from_entry(
        conn: &impl GenericClient,
        entry: &i32
    ) -> error::Result<Vec<ComposedEntryComment>> {
        Ok(conn.query(
            "\
            select entry_comments.id, \
                   entry_comments.entry, \
                   entry_comments.owner, \
                   entry_comments.comment, \
                   entry_comments.created, \
                   entry_comments.updated, \
                   users.id, \
                   users.username \
            from entry_comments \
            join users on entry_comments.owner = users.id \
            where entry_comments.entry = $1",
            &[entry]
        )
        .await?
        .iter()
        .map(|row| ComposedEntryComment {
            user: users::UserBare {
                id: row.get(6),
                username: row.get(7)
            },
            comment: entry_comments::EntryComment {
                id: row.get(0),
                entry: row.get(1),
                owner: row.get(2),
                comment: row.get(3),
                created: row.get(4),
                updated: row.get(5)
            }
        })
        .collect())
    }
    
}

#[derive(Serialize, Deserialize)]
pub struct ComposedUser {
    pub user: users::User,
    pub data: user_data::UserData,
}

#[derive(Serialize, Deserialize)]
pub struct ComposedUserAccess {
    pub user: users::User,
    pub access: user_access::UserAccess,
}

#[derive(Serialize, Deserialize)]
pub struct ComposedFullUser {
    pub user: users::User,
    pub data: user_data::UserData,
    pub access: Vec<ComposedUserAccess>
}

impl ComposedFullUser {

    // pub async fn find_from_id(
    //     conn: &impl GenericClient,
    //     id: &i32,
    //     access_as_owner: bool
    // ) -> error::Result<Option<ComposedFullUser>> {
    //     if let Some(user_data) = ComposedUser::find_from_id(conn, id).await? {
    //         let access = ComposedUserAccess::find(conn, id, access_as_owner).await?;

    //         Ok(Some(ComposedFullUser {
    //             user: user_data.user,
    //             data: user_data.data,
    //             access
    //         }))
    //     } else {
    //         Ok(None)
    //     }
    // }
    
}