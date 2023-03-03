use serde::{Serialize, Deserialize};
use tokio_postgres::GenericClient;

use crate::db::{error::Result, self};

/// checks to see if the given user has the roll with specified abilities.
/// 
/// can optionally check it against a specific user as well. example:
///
/// ```
/// use actix_web::{HttpRequest, Responder};
/// 
/// use crate::net::http::error;
/// use crate::security::{self, InitiatorLookup, Initiator};
/// use crate::state;
/// use crate::db::tables::permissions;
///
/// async fn handle_request(
///     db: state::WebDbState,
///     security: security::state::WebSecurityState,
/// ) -> error::Result<impl Responder> {
///     let conn = db.pool.get().await?;
///     let lookup = InitiatorLookup::from_request(&security, &*conn, &req).await?;
///     let initiator = lookup.try_into()?;
///     // optional user if checking
///     let user_id = 1;
/// 
///     if security::permissions::has_permission(
///         &*conn,
///         &initiator.user.id,
///         permissions::rolls::USERS_ENTRIES,
///         &[permissions::abilities::READ],
///         Some(&user_id)
///     ).await? {
///         // user has ability
///         Ok("you have permission")
///     } else {
///         Ok("you do not have permission")
///     }
/// }
/// ```
pub async fn has_permission(
    conn: &impl GenericClient,
    users_id: &i32,
    roll: &str,
    abilities: &[&str],
    resource: Option<&i32>,
) -> Result<bool> {
    if abilities.len() == 0 {
        return Ok(false);
    }

    let count = if let Some(resource_id) = resource {
        conn.execute(
            "\
            with subject_groups as (\
                select id \
                from groups \
                join group_users on \
                    groups.id = group_users.group_id \
                where group_users.users_id = $1 \
            ), resource_groups as (\
                select id \
                from groups \
                join group_users on \
                    groups.id = group_users.group_id \
                where group_users.users_id = $4 \
            ) \
            select subject_table, \
                   subject_id, \
                   roll, \
                   ability, \
                   resource_table, \
                   resource_id \
            from permissions \
            where roll = $2 and \
                  ability = any($3) and \
                  (\
                      (subject_table = 'groups' and subject_id in (select id from subject_groups)) or \
                      (subject_table = 'users' and subject_id = $1) \
                      ) and \
                  (\
                      (resource_table = 'groups' and resource_id in (select id from resource_groups)) or \
                      (resource_table = 'users' and resource_id = $4) \
                      )" ,
            &[users_id, &roll, &abilities, resource_id]
        ).await?
    } else {
        conn.execute(
            "\
            with user_groups as (\
                select id \
                from groups \
                join group_users on \
                    groups.id = group_users.group_id \
                where group_users.users_id = $1\
            ) \
            select subject_table, \
                   subject_id, \
                   roll, \
                   ability, \
                   resource_table, \
                   resource_id \
            from permissions \
            where roll = $2 and \
                  ability = any($3) and \
                  (\
                      (subject_table = 'groups' and subject_id in (select id from user_groups)) or \
                      (subject_table = 'users' and subject_id = $1)\
                      )",
            &[users_id, &roll, &abilities],
        )
        .await?
    };

    log::debug!("total permissions found: {}", count);

    Ok(count != 0)
}

/// common struct for specifying a subjects permission
/// 
/// to be given to update_subject_permissions so the subject table and id are
/// not necessary since the function takes those as arguments
#[derive(Serialize, Deserialize, Clone)]
pub struct PermissionJson {
    roll: String,
    ability: String,
    resource_table: Option<String>,
    resource_id: Option<i32>
}

/// return result from update_subject_permissions
/// 
/// the fields indicate the errors and what the permission was that caused it
#[derive(Serialize, Deserialize)]
pub struct FailedPermissions {
    unknown_roll: Vec<PermissionJson>,
    invalid_ability: Vec<PermissionJson>,
    unknown_resource_tables: Vec<PermissionJson>,
    resource_id_not_found: Vec<PermissionJson>,
    resource_not_allowed: Vec<PermissionJson>,
    same_as_subject: Vec<PermissionJson>,
}

/// updates a given subjects permissions to be the list given.
/// 
/// this is a fairly large process that is common between updating a group and 
/// user permssion list. it goes through a diff process of sorts to make the 
/// list of permissions for the subject and adjusts it to be like the list 
/// given.
pub async fn update_subject_permissions(
    conn: &impl GenericClient,
    subject_table: &str,
    subject_id: &i32,
    permissions: Vec<PermissionJson>
) -> Result<Option<FailedPermissions>> {
    let mut first = true;
    let mut invalid = false;
    let rolls = db::tables::permissions::RollDictionary::new();
    // error collection
    let mut failed = FailedPermissions {
        unknown_roll: Vec::new(),
        invalid_ability: Vec::new(),
        unknown_resource_tables: Vec::new(),
        resource_id_not_found: Vec::new(),
        resource_not_allowed: Vec::new(),
        same_as_subject: Vec::new(),
    };
    // database
    let mut query = "insert into permissions (subject_table, subject_id, roll, ability, resource_table, resource_id) values".to_owned();
    let mut value_sql = String::new();
    let mut query_params = db::query::QueryParams::with_capacity(2);
    query_params.push(&subject_table);
    query_params.push(&subject_id);

    for index in 0..permissions.len() {
        if !invalid {
            if first {
                first = false;
            } else {
                value_sql.push(',');
            }

            value_sql.push_str("($1,$2,");
        }

        let permission = &permissions[index];
        let Some(roll_data) = rolls.get_roll(&permission.roll) else {
            // roll does not exist
            failed.unknown_roll.push(permission.clone());
            invalid = true;
            value_sql.clear();
            continue;
        };

        if !invalid {
            let roll_p = query_params.push(&permission.roll).to_string();
            value_sql.push('$');
            value_sql.push_str(&roll_p);
            value_sql.push(',');
        }

        if !roll_data.check_ability(&permission.ability) {
            // invalid ability for given roll
            failed.invalid_ability.push(permission.clone());
            invalid = true;
            value_sql.clear();
            continue;
        } else if !invalid {
            let ability_p = query_params.push(&permission.ability).to_string();
            value_sql.push('$');
            value_sql.push_str(&ability_p);
            value_sql.push(',');
        }

        if permission.resource_table.is_some() && permission.resource_id.is_some() {
            if !roll_data.allow_resource {
                // given a specific resource but not allowed for given roll
                failed.resource_not_allowed.push(permission.clone());
                invalid = true;
                value_sql.clear();
                continue;
            }

            let resource_table = permission.resource_table.as_ref().unwrap();
            let resource_id = permission.resource_id.as_ref().unwrap();

            if resource_table == subject_table && resource_id == subject_id {
                failed.same_as_subject.push(permission.clone());
                invalid = true;
                value_sql.clear();
                continue;
            }

            match resource_table.as_str() {
                db::tables::permissions::tables::USERS => {
                    let check = conn.execute(
                        "select id from users where id = $1",
                        &[resource_id]
                    ).await?;

                    if check != 1 {
                        // resource not found
                        failed.resource_id_not_found.push(permission.clone());
                        invalid = true;
                        value_sql.clear();
                        continue;
                    }
                },
                db::tables::permissions::tables::GROUPS => {
                    // check to make sure that id exists
                    let check = conn.execute(
                        "select id from groups where id = $1",
                        &[resource_id]
                    ).await?;

                    if check != 1 {
                        // resource not found
                        failed.resource_id_not_found.push(permission.clone());
                        invalid = true;
                        value_sql.clear();
                        continue;
                    }
                },
                _ => {
                    // unknown table and needs to be delt with
                    failed.unknown_resource_tables.push(permission.clone());
                    invalid = true;
                    value_sql.clear();
                    continue;
                }
            }

            if !invalid {
                let resource_table_p = query_params.push(resource_table).to_string();
                let resource_id_p = query_params.push(resource_id).to_string();
                value_sql.push('$');
                value_sql.push_str(&resource_table_p);
                value_sql.push_str(",$");
                value_sql.push_str(&resource_id_p);
                value_sql.push(')');
            }
        } else if !invalid {
            value_sql.push_str("null,null)");
        }

        if !invalid {
            query.push_str(&value_sql);
            value_sql.clear();
        }
    }

    if invalid {
        return Ok(Some(failed))
    }

    query.push_str("on conflict on constraint unique_permissions do nothing returning id");

    let query_result = conn.query(&query, query_params.slice()).await?;
    let mut returned_ids: Vec<i32> = Vec::with_capacity(query_result.len());

    for row in query_result {
        returned_ids.push(row.get(0));
    }

    conn.execute(
        "\
        delete from permissions \
        where subject_table = $1 and \
              subject_id = $2 and \
              id <> all($2)",
        &[&subject_table, &subject_id, &returned_ids]
    ).await?;

    Ok(None)
}
