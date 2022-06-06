use tokio_postgres::GenericClient;

use crate::db;

use crate::response::error;

pub async fn get_via_id(
    conn: &impl GenericClient,
    id: &i32
) -> error::Result<db::global_custom_fields::GlobalCustomField> {
    if let Some(field) = db::global_custom_fields::find_from_id(conn, id).await? {
        Ok(field)
    } else {
        Err(error::ResponseError::GlobalCustomFieldNotFound(*id))
    }
}