use tokio_postgres::{Client, GenericClient, Error as PGError};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    id: i32,
    username: String,
    email: Option<String>
}

pub async fn insert(
    client: &impl GenericClient,
    username: &String,
    hash: &String,
    email: &Option<String>
) -> Result<User, PGError> {
    let result = client.query_one(
        r#"
        insert into users (level, username, hash, email) values
        (2, $1, $2, $3)
        returning id,
                  username,
                  email
        "#,
        &[&username, &hash, &email]
    ).await?;

    Ok(User {
        id: result.get(0),
        username: result.get(1),
        email: result.get(2)
    })
}

pub async fn check_username_email(
    client: &Client,
    username: &String,
    email: &Option<String>
) -> Result<(bool, bool), PGError> {
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

impl User {

    pub fn create(id: i32, username: String, email: Option<String>) -> Self {
        User { id, username, email }
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_id_ref(&self) -> &i32 {
        &self.id
    }
    
}