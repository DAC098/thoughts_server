use std::collections::{HashMap};

use tokio_postgres::{GenericClient};
use serde::{Serialize, Deserialize};

use crate::db::{
    error,
    users,
    user_data,
    user_access,
    entries,
    entry_markers,
    custom_field_entries,
    text_entries,
    entries2tags,
    entry_comments,
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

impl ComposedUser {

    pub async fn find_from_id(
        conn: &impl GenericClient,
        id: &i32
    ) -> error::Result<Option<ComposedUser>> {
        if let Some(row) = conn.query_opt(
            "\
            select users.id, \
                   users.username, \
                   users.email, \
                   users.email_verified, \
                   users.level, \
                   user_data.prefix, \
                   user_data.suffix, \
                   user_data.first_name, \
                   user_data.last_name, \
                   user_data.middle_name, \
                   user_data.dob \
            from users \
            join user_data on users.id = user_data.owner \
            where users.id = $1",
            &[id]
        ).await? {
            Ok(Some(ComposedUser {
                user: users::User {
                    id: *id,
                    username: row.get(1),
                    email: row.get(2),
                    email_verified: row.get(3),
                    level: row.get(4)
                },
                data: user_data::UserData {
                    owner: *id,
                    prefix: row.get(5),
                    suffix: row.get(6),
                    first_name: row.get(7),
                    last_name: row.get(8),
                    middle_name: row.get(9),
                    dob: row.get(10)
                }
            }))
        } else {
            Ok(None)
        }
    }
    
}

#[derive(Serialize, Deserialize)]
pub struct ComposedUserAccess {
    pub user: users::User,
    pub access: user_access::UserAccess,
}

impl ComposedUserAccess {

    pub async fn find(
        conn: &impl GenericClient,
        id: &i32,
        as_owner: bool
    ) -> error::Result<Vec<ComposedUserAccess>> {
        let query_str = format!(
            "\
            select users.id, \
                   users.username, \
                   users.email, \
                   users.email_verified, \
                   users.level, \
                   user_access.owner, \
                   user_access.ability, \
                   user_access.allowed_for \
            from users \
            join user_access on users.id = user_access.{} \
            where user_access.{} = $1",
            if as_owner { "allowed_for" } else { "owner" },
            if as_owner { "owner" } else { "allowed_for" }
        );

        Ok(
            conn.query(query_str.as_str(), &[id]).await?
            .iter()
            .map(|row| ComposedUserAccess {
                user: users::User {
                    id: row.get(0),
                    username: row.get(1),
                    email: row.get(2),
                    email_verified: row.get(3),
                    level: row.get(4)
                },
                access: user_access::UserAccess {
                    owner: row.get(5),
                    ability: row.get(6),
                    allowed_for: row.get(7)
                }
            })
            .collect()
        )
    }
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