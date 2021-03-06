use tokio_postgres::{GenericClient};
use crate::db::{custom_fields};

use crate::response::error;

pub async fn get_via_id(
    conn: &impl GenericClient,
    id: i32,
    initiator_opt: Option<i32>,
) -> error::Result<custom_fields::CustomField> {
    if let Some(field) = custom_fields::find_from_id(conn, &id).await? {
        if let Some(initiator) = initiator_opt {
            if field.owner != initiator {
                return Err(error::ResponseError::PermissionDenied(
                    format!("you do not haver permission to create a custom field entry using this field id: {}", field.owner)
                ))
            }
        }

        Ok(field)
    } else {
        Err(error::ResponseError::CustomFieldNotFound(id))
    }
}