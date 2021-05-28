use tokio_postgres::{Client, GenericClient, Error as PGError};

use crate::db::users;

pub async fn find_token_user(
    client: &Client,
    token: uuid::Uuid
) -> Result<Option<users::User>, PGError> {
    let result = client.query(
        r#"
        select users.id, 
               users.username,
               users.full_name,
               users.email,
               users.email_verified,
               users.level
        from user_sessions 
        join users on user_sessions.owner = users.id 
        where token = $1
        "#,
        &[&token]
    ).await?;

    if result.len() == 0 {
        Ok(None)
    } else {
        Ok(Some(users::User {
            id: result[0].get(0),
            username: result[0].get(1),
            full_name: result[0].get(2),
            email: result[0].get(3),
            email_verified: result[0].get(4),
            level: result[0].get(5)
        }))
    }
}

pub async fn insert(
    client: &impl GenericClient,
    token: uuid::Uuid,
    owner: i32
) -> Result<bool, PGError> {
    let result = client.query_one(
        r#"
        insert into user_sessions (token, owner) values
        ($1, $2)
        returning token
        "#,
        &[&token, &owner]
    ).await?;

    if result.len() == 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn delete(
    client: &impl GenericClient,
    token: uuid::Uuid,
) -> Result<u64, PGError> {
    Ok(client.execute(
        "delete from user_sessions where token = $1",
        &[&token]
    ).await?)
}