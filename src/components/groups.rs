use std::collections::HashSet;

use tokio_postgres::GenericClient;

use crate::net::http::error;
use crate::db;

/// updates a groups user list
/// 
/// pretty much it, the returned result will either be None indicating that
/// all the provided users were valid or Some indicating that there was some
/// invalid users provided
pub async fn update_group_users(
    conn: &impl GenericClient, 
    group_id: &i32, 
    user_ids: Vec<i32>
) -> error::Result<Option<Vec<i32>>> {
    let users_check = conn.query(
        "select id from users where id = any($1)",
        &[&user_ids]
    ).await?;
    let mut id_set: HashSet<i32> = HashSet::with_capacity(users_check.len());
    let mut invalid_ids = Vec::with_capacity(user_ids.len());
    let mut valid_ids = Vec::with_capacity(users_check.len());

    for row in users_check {
        let id = row.get(0);
        id_set.insert(id);
        valid_ids.push(id);
    }

    for id in user_ids {
        if !id_set.contains(&id) {
            invalid_ids.push(id);
        }
    }

    if invalid_ids.len() > 0 {
        return Ok(Some(invalid_ids));
    }

    let mut query = "insert into group_users (group_id, users_id) values ".to_owned();
    let mut params = db::query::QueryParams::with_capacity(valid_ids.len() + 1);
    params.push(&group_id);

    for i in 0..valid_ids.len() {
        let key = params.push(&valid_ids[i]).to_string();
        
        if i == 0 {
            query.reserve(key.len() + 6);
        } else {
            query.reserve(key.len() + 7);
            query.push(',');
        }

        query.push_str("($1,$");
        query.push_str(&key);
        query.push_str(")");
    }

    query.push_str(" on conflict (users_id, group_id) do nothing");

    conn.execute(query.as_str(), params.slice()).await?;
    conn.execute(
        "delete from group_users where group_id = $1 and users_id <> all($2)",
        &[&group_id, &valid_ids]
    ).await?;

    Ok(None)
}