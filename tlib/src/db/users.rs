use std::convert::{From};

use serde::{Serialize, Deserialize};
use tokio_postgres::{GenericClient};

#[repr(i32)]
pub enum Level {
    Admin = 1,
    Manager = 10,
    User = 20
}

use crate::db::error;

use error::Result;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub level: i32,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool
}

#[derive(Serialize, Deserialize)]
pub struct UserBare {
    pub id: i32,
    pub username: String,
    pub full_name: Option<String>
}

impl From<User> for UserBare {

    fn from(user: User) -> Self {
        UserBare {
            id: user.id,
            username: user.username,
            full_name: user.full_name
        }
    }

}

impl From<&User> for UserBare {

    fn from(user: &User) -> Self {
        UserBare {
            id: user.id.clone(),
            username: user.username.clone(),
            full_name: user.full_name.clone()
        }
    }

}

pub async fn check_username_email(
    client: &impl GenericClient,
    username: &String,
    email: &String
) -> Result<(bool, bool)> {
    let result = client.query(
        "\
        select username = $1 as same_username, \
               email = $2 as same_email \
        from users \
        where username = $1 or email = $2",
        &[&username, &email]
    ).await?;
    let mut found_username = false;
    let mut found_email = false;

    for row in result {
        if row.get::<usize,bool>(0) {
            found_username = true;
        }

        if row.get::<usize,bool>(1) {
            found_email = true;
        }
    }

    Ok((found_username, found_email))
}

pub async fn find_from_id(
    client: &impl GenericClient,
    id: &i32
) -> Result<Option<User>> {
    if let Some(row) = client.query_opt(
        "\
        select id, \
               username, \
               full_name, \
               email, \
               email_verified, \
               level \
        from users \
        where id = $1",
        &[&id]
    ).await? {
        Ok(Some(User {
            id: row.get(0),
            username: row.get(1),
            full_name: row.get(2),
            email: row.get(3),
            email_verified: row.get(4),
            level: row.get(5)
        }))
    } else {
        Ok(None)
    }
}

pub async fn find_from_session_token(
    conn: &impl GenericClient,
    token: uuid::Uuid
) -> Result<Option<User>> {
    if let Some(row) = conn.query_opt(
        "\
        select id, \
               username, \
               full_name, \
               email, \
               email_verified, \
               level \
        from users \
        where id = (\
            select owner \
            from user_sessions \
            where token = $1\
        )",
        &[&token]
    ).await? {
        Ok(Some(User {
            id: row.get(0),
            username: row.get(1),
            full_name: row.get(2),
            email: row.get(3),
            email_verified: row.get(4),
            level: row.get(5)
        }))
    } else {
        Ok(None)
    }
}