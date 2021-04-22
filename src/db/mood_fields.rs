use tokio_postgres::{GenericClient, Error as PGError};

pub struct MoodField {
    id: i32,
    name: String,
    owner: i32,
    minimum: Option<i32>,
    maximum: Option<i32>,
    is_range: bool
}

pub async fn find_id(
    conn: &impl GenericClient,
    id: i32
) -> Result<Option<MoodField>, PGError> {
    let result = conn.query(
        r#"
        select id, 
               name, owner,
               minimum, maximum,
               is_range, 
               comment 
        from mood_fields where id = $1
        "#,
        &[&id]
    ).await?;

    if result.len() == 0 {
        Ok(None)
    } else {
        Ok(Some(MoodField {
            id: result[0].get(0),
            name: result[0].get(1),
            owner: result[0].get(2),
            minimum: result[0].get(3),
            maximum: result[0].get(4),
            is_range: result[0].get(5)
        }))
    }
}

impl MoodField {

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    
    pub fn get_owner(&self) -> i32 {
        self.owner
    }

    pub fn get_minimum(&self) -> Option<i32> {
        self.minimum
    }

    pub fn get_maximum(&self) -> Option<i32> {
        self.maximum
    }

    pub fn get_is_range(&self) -> bool {
        self.is_range
    }
}