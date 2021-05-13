use tokio_postgres::{GenericClient};

use crate::error;

pub async fn is_owner_for_entry(
    conn: &impl GenericClient,
    entry_id: i32,
    initiator: i32,
) -> error::Result<()> {
    let owner_result = conn.query(
        "select owner from entries where id = $1", 
        &[&entry_id]
    ).await?;

    if owner_result.len() == 0 {
        return Err(error::ResponseError::EntryNotFound(entry_id));
    }

    if owner_result[0].get::<usize, i32>(0) != initiator {
        return Err(error::ResponseError::PermissionDenied(
            "you don't have permission to modify this users entry".to_owned()
        ));
    }
    
    Ok(())
}

pub async fn is_owner_for_mood_field(
    conn: &impl GenericClient,
    field_id: i32,
    owner: i32,
) -> error::Result<()> {
    let rows = conn.query(
        "select owner from mood_fields where id = $1",
        &[&field_id]
    ).await?;

    if rows.len() == 0 {
        return Err(error::ResponseError::MoodFieldNotFound(field_id));
    }

    if rows[0].get::<usize, i32>(0) != owner {
        return Err(error::ResponseError::PermissionDenied(
            "you don't have permission to modify this users mood field".to_owned()
        ));
    }

    Ok(())
}

pub async fn permission_to_read(
    conn: &impl GenericClient,
    initiator: i32,
    user: i32
) -> error::Result<()> {
    let result = conn.query(
        "select ability, allowed_for from user_access where owner = $1",
        &[&user]
    ).await?;

    if result.len() > 0 {
        for row in result {
            let ability: String = row.get(0);
            let allowed_for: i32 = row.get(1);

            if ability.eq("r") && initiator == allowed_for {
                return Ok(());
            }
        }

        Err(error::ResponseError::PermissionDenied(
            "you do not have permission to read this users information".to_owned()
        ))
    } else {
        Err(error::ResponseError::PermissionDenied(
            "no ability was found for the requested user".to_owned()
        ))
    }
}