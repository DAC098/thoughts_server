use std::fmt;

use tokio_postgres::{Client, Error as PGError};
use chrono::NaiveDate;

pub struct Entry {
    id: i32,
    created: NaiveDate,
    owner: i32
}

impl Entry {

    pub async fn find_id(
        client: &Client,
        id: i32
    ) -> Result<Entry, PGError> {
        let result = client.query_one(
            "SELECT * FROM entries WHERE id = $1",
            &[&id]
        ).await?;

        Ok(Entry {
            id: result.get(0),
            created: result.get(1),
            owner: result.get(2)
        })
    }

    pub async fn find(
        client: &Client
    ) -> Result<Vec<Entry>, PGError> {
        let result = client.query("SELECT * FROM entries", &[]).await?;
        let mut rtn: Vec<Entry> = vec!();

        for row in result {
            &rtn.push(Entry {
                id: row.get(0),
                created: row.get(1),
                owner: row.get(2)
            });
        }

        Ok(rtn)
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_created(&self) -> NaiveDate {
        self.created
    }

    pub fn get_owner(&self) -> i32 {
        self.owner
    }
    
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Entry {{ id: {}, created: {}, owner: {} }}", self.id, self.created, self.owner)
    }
}