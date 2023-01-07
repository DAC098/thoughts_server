use std::collections::HashMap;
use serde::Serialize;
use tokio_postgres::GenericClient;

use crate::db::error::Result;

pub mod tables {
    pub const GROUPS: &str = "groups";
    pub const USERS: &str = "users";
}

pub mod rolls {
    pub const ENTRIES: &str = "entries";
    pub const USERS: &str = "users";
    pub const USERS_ENTRIES: &str = "users/etnries";
    pub const USERS_ENTRIES_COMMENTS: &str = "users/entries/comments";
    pub const GROUPS: &str = "groups";
    pub const GLOBAL_TAGS: &str = "global/tags";
    pub const GLOBAL_CUSTOM_FIELDS: &str = "global/custom_fields";
}

pub mod abilities {
    pub const READ: &str = "r";
    pub const READ_WRITE: &str = "rw";
}

pub struct RollData {
    pub abilities: Vec<&'static str>,
    pub allow_resource: bool
}

impl RollData {
    pub fn check_ability(&self, ability: &str) -> bool {
        for allowed in &self.abilities {
            if ability == *allowed {
                return true;
            }
        }

        false
    }
}

pub struct RollDictionary {
    mapping: HashMap<&'static str, RollData>
}

impl RollDictionary {
    pub fn new() -> Self {
        let mapping = {
            let mut rtn = HashMap::with_capacity(7);
            rtn.insert(rolls::ENTRIES, RollData {
                abilities: [
                    abilities::READ,
                    abilities::READ_WRITE
                ].into(),
                allow_resource: false
            });
            rtn.insert(rolls::USERS, RollData {
                abilities: [
                    abilities::READ,
                    abilities::READ_WRITE
                ].into(),
                allow_resource: true
            });
            rtn.insert(rolls::USERS_ENTRIES, RollData {
                abilities: [
                    abilities::READ
                ].into(),
                allow_resource: false
            });
            rtn.insert(rolls::USERS_ENTRIES_COMMENTS, RollData {
                abilities: [
                    abilities::READ,
                    abilities::READ_WRITE
                ].into(),
                allow_resource: false
            });
            rtn.insert(rolls::GROUPS, RollData {
                abilities: [
                    abilities::READ,
                    abilities::READ_WRITE
                ].into(),
                allow_resource: true
            });
            rtn.insert(rolls::GLOBAL_TAGS, RollData {
                abilities: [
                    abilities::READ,
                    abilities::READ_WRITE
                ].into(),
                allow_resource: false
            });
            rtn.insert(rolls::GLOBAL_CUSTOM_FIELDS, RollData { 
                abilities: [
                    abilities::READ,
                    abilities::READ_WRITE
                ].into(), 
                allow_resource: false
            });

            rtn
        };

        RollDictionary { mapping }
    }

    pub fn get_roll(&self, roll: &str) -> Option<&RollData> {
        self.mapping.get(roll)
    }
}

#[derive(Serialize)]
pub struct Permission {
    id: i32,
    subject_table: String,
    subject_id: i32,
    roll: String,
    ability: String,
    resource_table: Option<String>,
    resource_id: Option<i32>
}

pub async fn _find_from_subject(conn: &impl GenericClient, table: &str, id: &i32) -> Result<Vec<Permission>> {
    Ok(conn.query(
        "\
        select id, \
               subject_table, \
               subject_id, \
               roll, \
               ability, \
               resource_table, \
               resource_id \
        from permissions \
        where subject_table = $1 and \
              subject_id = $2",
        &[&table, id]
    ).await?
    .iter()
    .map(|row| Permission {
        id: row.get(0),
        subject_table: row.get(1),
        subject_id: row.get(2),
        roll: row.get(3),
        ability: row.get(4),
        resource_table: row.get(5),
        resource_id: row.get(6)
    })
    .collect())
}

pub async fn _find_from_resource(conn: &impl GenericClient, table: &str, id: &i32) -> Result<Vec<Permission>> {
    Ok(conn.query(
        "\
        select id, \
               subject_table, \
               subject_id, \
               roll, \
               ability, \
               resource_table, \
               resource_id \
        from permissions \
        where resource_table = $1 and \
              resource_id = $2",
            &[&table, id]
    ).await?
    .iter()
    .map(|row| Permission {
        id: row.get(0),
        subject_table: row.get(1),
        subject_id: row.get(2),
        roll: row.get(3),
        ability: row.get(4),
        resource_table: row.get(5),
        resource_id: row.get(6)
    })
    .collect())
}

pub async fn _find_user_permissions(conn: &impl GenericClient, users_id: &i32) -> Result<Vec<Permission>> {
    Ok(conn.query(
        "\
        with user_groups ( \
            select groups.id \
            from groups \
            join group_users on \
                groups.id = group_users.group_id \
            where group_users.users_id = $1 \
        ) \
        select id, \
               subject_table, \
               subject_id, \
               roll, \
               ability, \
               resource_table, \
               resource_id \
        from permissions \
        where (subject_table = 'groups' and subject_id in user_groups.id) or \
              (subject_table = 'users' and subject_id = $1)",
        &[users_id]
    ).await?
    .iter()
    .map(|row| Permission {
        id: row.get(0),
        subject_table: row.get(1),
        subject_id: row.get(2),
        roll: row.get(3),
        ability: row.get(4),
        resource_table: row.get(5),
        resource_id: row.get(6)
    })
    .collect())
}