use tokio_postgres::{Client, Error as PGError};
use serde::{Serialize, Deserialize};

use crate::db::users;

pub struct UserSession {
    token: uuid::Uuid,
    owner: i32
}

impl UserSession {

    pub async fn find_token_owner(
        client: &Client,
        token: uuid::Uuid
    ) -> Result<Option<i32>, PGError> {
        let result = client.query(
            "select owner from user_sessions where token = $1",
            &[&token]
        ).await?;

        if result.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(result[0].get(0)))
        }
    }

    pub async fn find_token_user(
        client: &Client,
        token: uuid::Uuid
    ) -> Result<Option<users::User>, PGError> {
        let result = client.query(
            "select users.id, users.username, users.email from user_sessions join users on user_sessions.owner = users.id where token = $1",
            &[&token]
        ).await?;

        if result.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(users::User::create(
                result[0].get(0),
                result[0].get(1),
                result[0].get(2)
            )))
        }
    }

    pub async fn insert(
        client: &Client,
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

    pub fn get_token(&self) -> uuid::Uuid {
        self.token
    }

    pub fn get_owner(&self) -> i32 {
        self.owner
    }
    
}