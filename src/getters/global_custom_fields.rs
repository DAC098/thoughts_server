use tokio_postgres::{GenericClient};

use crate::json;
use crate::response::{error};

pub async fn get_via_id(
    conn: &impl GenericClient,
    id: i32
) -> error::Result<json::GlobalCustomFieldJson> {
    let result = conn.query(
        r#"select id, \
                  name, \
                  comment, \
                  config \
           from global_custom_fields
           where id = $1"#,
        &[&id]
    ).await?;

    if result.len() == 1 {
        Ok(json::GlobalCustomFieldJson {
            id: id,
            name: result[0].get(1),
            comment: result[0].get(2),
            config: serde_json::from_value(result[0].get(3))?
        })
    } else {
        Err(error::ResponseError::GlobalCustomFieldNotFound(id))
    }
}