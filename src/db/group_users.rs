use tokio_postgres::GenericClient;

use crate::db::error::Result;

pub struct GroupUser {
    users_id: i32,
    group_id: i32
}

pub async fn find_users_id(conn: &impl GenericClient, users_id: &i32) -> Result<Vec<GroupUser>> {
    Ok(conn.query(
        "\
        select users_id, \
               group_id \
        from group_users \
        where users_id = $1",
        &[
            users_id
        ]
    ).await?
    .iter()
    .map(|v|
        GroupUser {
            users_id: v.get(0),
            group_id: v.get(1)
        }
    )
    .collect())
}

pub async fn find_group_id(conn: &impl GenericClient, group_id: &i32) -> Result<Vec<GroupUser>> {
    Ok(conn.query(
        "\
        select users_id, \
               group_id
        from group_users \
        where group_id = $1",
        &[
            group_id
        ]
    ).await?
    .iter()
    .map(|v| GroupUser {
        users_id: v.get(0),
        group_id: v.get(1)
    })
    .collect())
}