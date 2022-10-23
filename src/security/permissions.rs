use tokio_postgres::GenericClient;

use crate::db::error::Result;

/// checks to see if the given user has the roll with specified abilities.
/// can optionally check it against a specific user as well
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
