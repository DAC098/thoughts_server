use tokio_postgres::{GenericClient};
use serde::{Serialize, Deserialize};

use crate::db::error;

#[derive(Serialize, Deserialize)]
pub struct UserSession {
    pub token: uuid::Uuid,
    pub owner: i32,
    pub expires: chrono::DateTime<chrono::Utc>
}

pub async fn find_from_owner(
    conn: &impl GenericClient,
    owner: i32
) -> error::Result<Vec<UserSession>> {
    Ok(
        conn.query(
            "select token, owner, expires from user_sessions where owner = $1",
            &[&owner]
        )
        .await?
        .iter()
        .map(|row| UserSession {
            token: row.get(0),
            owner: row.get(1),
            expires: row.get(2)
        })
        .collect()
    )
}

pub async fn find_from_token(
    conn: &impl GenericClient,
    token: uuid::Uuid
) -> error::Result<Option<UserSession>> {
    let result = conn.query(
        "select token, owner, expires from user_sessions where token = $1",
        &[&token]
    ).await?;

    if result.len() == 0 {
        Ok(None)
    } else {
        Ok(Some(UserSession {
            token: result[0].get(0),
            owner: result[0].get(1),
            expires: result[0].get(2)
        }))
    }
}

pub async fn insert(
    conn: &impl GenericClient,
    token: uuid::Uuid,
    owner: i32
) -> error::Result<bool> {
    let result = conn.execute(
        "\
        insert into user_sessions (token, owner) values \
        ($1, $2)
        ",
        &[&token, &owner]
    ).await?;

    Ok(result == 1)
}

pub async fn delete(
    conn: &impl GenericClient,
    token: uuid::Uuid,
) -> error::Result<bool> {
    Ok(conn.execute(
        "delete from user_sessions where token = $1", 
        &[&token]
    ).await? == 1)
}