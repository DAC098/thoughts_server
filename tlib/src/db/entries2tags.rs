use tokio_postgres::{GenericClient};

use crate::db::{error};

pub async fn find_id_from_entry(
    conn: &impl GenericClient,
    entry_id: &i32
) -> error::Result<Vec<i32>> {
    Ok(
        conn.query(
            "select tag from entries2tags where entry = $1",
            &[entry_id]
        )
        .await?
        .iter()
        .map(|row| row.get::<usize, i32>(0))
        .collect()
    )
}