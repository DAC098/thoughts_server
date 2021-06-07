use tokio_postgres::{GenericClient};
use serde::{Serialize, Deserialize};

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

pub async fn check_username_email(
    client: &impl GenericClient,
    username: &String,
    email: &String
) -> Result<(bool, bool)> {
    let result = client.query(
        r#"
        select username = $1 as same_username, 
               email = $2 as same_email 
        from users 
        where username = $1 or email = $2
        "#,
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

pub async fn find_via_id(
    client: &impl GenericClient,
    id: i32
) -> Result<Option<User>> {
    let result = client.query(
        r#"
        select id, 
               username, 
               full_name, 
               email, 
               email_verified,
               level 
        from users 
        where id = $1
        "#,
        &[&id]
    ).await?;

    if result.len() == 1 {
        Ok(Some( User {
            id: result[0].get(0),
            username: result[0].get(1),
            full_name: result[0].get(2),
            email: result[0].get(3),
            email_verified: result[0].get(4),
            level: result[0].get(5)
        }))
    } else {
        Ok(None)
    }
}

impl User {

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_id_ref(&self) -> &i32 {
        &self.id
    }
    
}