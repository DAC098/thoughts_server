use tokio_postgres::{GenericClient};

use crate::db;
use crate::{net::http::error, request::Initiator};

pub async fn is_owner_for_entry(
    conn: &impl GenericClient,
    entry_id: &i32,
    initiator: &i32,
) -> error::Result<()> {
    let owner_result = conn.query(
        "select owner from entries where id = $1", 
        &[&entry_id]
    ).await?;

    if owner_result.len() == 0 {
        return Err(error::ResponseError::EntryNotFound(*entry_id));
    }

    if owner_result[0].get::<usize, i32>(0) != *initiator {
        return Err(error::ResponseError::PermissionDenied(
            "you don't have permission to modify this users entry".to_owned()
        ));
    }
    
    Ok(())
}

pub async fn is_owner_for_custom_field(
    conn: &impl GenericClient,
    field_id: &i32,
    owner: &i32,
) -> error::Result<()> {
    let rows = conn.query(
        "select owner from custom_fields where id = $1",
        &[field_id]
    ).await?;

    if rows.len() == 0 {
        return Err(error::ResponseError::CustomFieldNotFound(*field_id));
    }

    if rows[0].get::<usize, i32>(0) != *owner {
        return Err(error::ResponseError::PermissionDenied(
            "you don't have permission to modify this users custom field".to_owned()
        ));
    }

    Ok(())
}

pub async fn is_owner_for_tag(
    conn: &impl GenericClient,
    tag_id: &i32,
    owner: &i32
) -> error::Result<()> {
    let rows = conn.query(
        "select owner from tags where id = $1",
        &[tag_id]
    ).await?;

    if rows.len() == 0 {
        Err(error::ResponseError::TagNotFound(*tag_id))
    } else {
        if rows[0].get::<usize, i32>(0) != *owner {
            Err(error::ResponseError::PermissionDenied(
                "you don't have permission to modify this users tag".to_owned()
            ))
        } else {
            Ok(())
        }
    }
}

pub async fn permission_to_read(
    conn: &impl GenericClient,
    initiator: &i32,
    user: &i32
) -> error::Result<()> {
    let result = conn.query(
        "select ability, allowed_for from user_access where owner = $1",
        &[initiator]
    ).await?;

    if result.len() > 0 {
        for row in result {
            let ability: String = row.get(0);
            let allowed_for: i32 = row.get(1);

            if ability.eq("r") && *user == allowed_for {
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

pub fn is_admin(initiator: &Initiator) -> error::Result<()> {
    if initiator.user.level != (db::users::Level::Admin as i32) {
        Err(error::ResponseError::PermissionDenied(
            format!("you are not an administrator")
        ))
    } else {
        Ok(())
    }
}

pub async fn is_owner_of_entry(
    conn: &impl GenericClient,
    owner: &i32,
    entry: &i32,
) -> error::Result<()> {
    let result = conn.query(
        "select owner from entries where id = $1",
        &[entry]
    ).await?;

    if result.len() == 1 {
        let entry_owner: i32 = result[0].get(0);

        if *owner == entry_owner {
            Ok(())
        } else {
            Err(error::ResponseError::PermissionDenied(
                format!("entry owner mis-match. requested entry is not owned by {}", *owner)
            ))
        }
    } else {
        Err(error::ResponseError::EntryNotFound(*entry))
    }
}