use tokio_postgres::GenericClient;

use crate::db::error;

#[derive(Clone)]
pub struct UserSession {
    pub token: String,
    pub owner: i32,
    pub dropped: bool,
    pub issued_on: chrono::DateTime<chrono::Utc>,
    pub expires: chrono::DateTime<chrono::Utc>,
    pub verified: bool,
    pub use_csrf: bool,
}

impl UserSession {

    // pub async fn find_from_owner(
    //     conn: &impl GenericClient,
    //     owner: i32
    // ) -> error::Result<Vec<UserSession>> {
    //     Ok(
    //         conn.query(
    //             "\
    //             select token, \
    //                    owner, \
    //                    dropped, \
    //                    issued_on, \
    //                    expires \
    //             from user_sessions \
    //             where owner = $1",
    //             &[&owner]
    //         )
    //         .await?
    //         .iter()
    //         .map(|row| UserSession {
    //             token: row.get(0),
    //             owner: row.get(1),
    //             dropped: row.get(2),
    //             issued_on: row.get(3),
    //             expires: row.get(4),
    //             use_csrf: row.get(5)
    //         })
    //         .collect()
    //     )
    // }

    pub async fn find_from_token(
        conn: &impl GenericClient,
        token: &str
    ) -> error::Result<Option<UserSession>> {
        if let Some(record) = conn.query_opt(
            "\
            select token, \
                   owner, \
                   dropped, \
                   issued_on, \
                   expires, \
                   verified, \
                   use_csrf \
            from user_sessions \
            where token = $1",
            &[&token]
        ).await? {
            Ok(Some(UserSession {
                token: record.get(0),
                owner: record.get(1),
                dropped: record.get(2),
                issued_on: record.get(3),
                expires: record.get(4),
                verified: record.get(5),
                use_csrf: record.get(6)
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn delete(&self, conn: &impl GenericClient) -> error::Result<u64> {
        let result = conn.execute(
            "delete from user_sessions where token = $1",
            &[&self.token]
        ).await?;

        Ok(result)
    }

    pub async fn insert(&self, conn: &impl GenericClient) -> error::Result<()> {
        conn.execute(
            "\
            insert into user_sessions (token, owner, dropped, issued_on, expires, use_csrf, verified) \
            values ($1, $2, $3, $4, $5, $6, $7)",
            &[
                &self.token,
                &self.owner,
                &self.dropped,
                &self.issued_on,
                &self.expires,
                &self.use_csrf,
                &self.verified,
            ]
        ).await?;

        Ok(())
    }

    // pub async fn delete(&self, conn: &impl GenericClient) -> error::Result<bool> {
    //     Self::delete_via_token(conn, &self.token).await
    // }
}

impl Default for UserSession {
    fn default() -> Self {
        let chrono_now = chrono::Utc::now();

        UserSession {
            token: String::new(), 
            owner: 0, 
            dropped: false, 
            issued_on: chrono_now.clone(), 
            expires: chrono_now,
            verified: false,
            use_csrf: false
        }
    }
}