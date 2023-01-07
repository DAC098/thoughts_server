use tokio_postgres::GenericClient;

use crate::db::tables::custom_fields;
use crate::net::http::error;

pub async fn get_via_id(
    conn: &impl GenericClient,
    id: &i32,
    initiator_opt: Option<&i32>,
) -> error::Result<custom_fields::CustomField> {
    if let Some(field) = custom_fields::find_from_id(conn, id).await? {
        if let Some(initiator) = initiator_opt {
            if field.owner != *initiator {
                return Err(error::build::permission_denied(
                    format!("you are not the owner of this custom field entry: {}", field.owner)
                ))
            }
        }

        Ok(field)
    } else {
        Err(error::build::custom_field_not_found(id))
    }
}