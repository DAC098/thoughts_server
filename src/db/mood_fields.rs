use tokio_postgres::{Client, Error as PGError};

pub struct MoodField {
    id: i32,
    name: String,
    owner: i32,
    is_range: bool,
    comment: Option<String>
}

impl MoodField {

    pub async fn find_id(
        conn: &Client,
        id: i32
    ) -> Result<Option<Self>, PGError> {
        let result = conn.query(
            "select id, name, owner, is_range, comment from mood_fields where id = $1",
            &[&id]
        ).await?;

        if result.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(MoodField {
                id: result[0].get(0),
                name: result[0].get(1),
                owner: result[0].get(2),
                is_range: result[0].get(3),
                comment: result[0].get(4)
            }))
        }
    }

    pub fn create(id: i32, name: String, owner: i32, is_range: bool, comment: Option<String>) -> Self {
        MoodField {
            id, name, owner, is_range, comment
        }
    }
    
    pub fn get_owner(&self) -> i32 {
        self.owner
    }

    pub fn get_is_range(&self) -> bool {
        self.is_range
    }
}