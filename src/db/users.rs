use tokio_postgres::{Client, Error as PGError};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    id: i32,
    username: String,
    email: Option<String>
}

impl User {

    pub async fn find_id(
        client: &Client,
        id: i32
    ) -> Result<Option<User>, PGError> {
        let result = client.query(
            r#"
            select id, username, email
            from users 
            where id = $1"#, 
            &[&id]
        ).await?;

        if result.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(User {
                id: result[0].get(0),
                username: result[0].get(1),
                email: result[0].get(4)
            }))
        }
    }

    pub async fn check_username_email(
        client: &Client,
        username: &String,
        email: &Option<String>
    ) -> Result<(bool, bool), PGError> {
        let result = client.query(
            "select username = $1 as same_username, email = $2 as same_email from users where username = $1 or email = $2",
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

    pub async fn insert(
        client: &Client,
        username: &String,
        hash: &String,
        email: &Option<String>
    ) -> Result<User, PGError> {
        let result = client.query_one(
            r#"
            insert into users (username, hash, email) values
            ($1, $2, $3)
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

    pub fn create(id: i32, username: String, email: Option<String>) -> Self {
        User { id, username, email }
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_id_ref(&self) -> &i32 {
        &self.id
    }

    pub fn get_username(&self) -> String {
        self.username.clone()
    }

    pub fn get_email(&self) -> Option<String> {
        match &self.email {
            Some(em) => Some(em.clone()),
            None => None
        }
    }
    
}