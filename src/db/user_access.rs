// use tokio_postgres::GenericClient;
use serde::{Serialize, Deserialize};

// use crate::db::error;

#[derive(Serialize, Deserialize)]
pub struct UserAccess {
    pub owner: i32,
    pub ability: String,
    pub allowed_for: i32
}

// pub async fn find(
//     conn: &impl GenericClient,
//     id: &i32,
//     as_owner: bool
// ) -> error::Result<Vec<UserAccess>> {
//     let query_str = format!(
//         "\
//         select owner, \
//                 ability, \
//                 allowed_for \
//         from user_access \
//         where {} = $1",
//         if as_owner { "owner" } else { "allowed_for" }
//     );

//     Ok(
//         conn.query(query_str.as_str(), &[id])
//         .await?
//         .iter()
//         .map(|row| UserAccess {
//             owner: row.get(0),
//             ability: row.get(1),
//             allowed_for: row.get(2)
//         })
//         .collect()
//     )
// }