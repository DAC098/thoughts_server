use tokio_postgres::GenericClient;

use crate::db::error;

pub struct UserSession {
    pub token: uuid::Uuid,
    pub owner: i32,
    pub dropped: bool,
    pub issued_on: chrono::DateTime<chrono::Utc>,
    pub expires: chrono::DateTime<chrono::Utc>,
    pub use_csrf: bool,
}

impl UserSession {

    pub fn new(owner: i32, issued_on: chrono::DateTime<chrono::Utc>, duration: chrono::Duration) -> UserSession {
        let expires = issued_on.checked_add_signed(duration).unwrap();

        UserSession {
            token: uuid::Uuid::new_v4(),
            owner,
            dropped: false,
            issued_on,
            expires,
            use_csrf: false,
        }
    }

    pub async fn find_from_owner(
        conn: &impl GenericClient,
        owner: i32
    ) -> error::Result<Vec<UserSession>> {
        Ok(
            conn.query(
                "\
                select token, \
                       owner, \
                       dropped, \
                       issued_on, \
                       expires \
                from user_sessions \
                where owner = $1",
                &[&owner]
            )
            .await?
            .iter()
            .map(|row| UserSession {
                token: row.get(0),
                owner: row.get(1),
                dropped: row.get(2),
                issued_on: row.get(3),
                expires: row.get(4),
                use_csrf: row.get(5)
            })
            .collect()
        )
    }

    pub async fn find_from_token(
        conn: &impl GenericClient,
        token: uuid::Uuid
    ) -> error::Result<Option<UserSession>> {
        if let Some(record) = conn.query_opt(
            "
            select token, \
                   owner, \
                   dropped, \
                   issued_on, \
                   expires, \
                   use_csrf
            from user_sessions \
            where token = $1",
            &[&token]
        ).await? {
            Ok(Some(UserSession {
                token: record.get(0),
                owner: record.get(1),
                dropped: record.get(2),
                issued_on: record.get(3),
                expires: record.get(2),
                use_csrf: record.get(4)
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_via_token(
        conn: &impl GenericClient,
        token: &uuid::Uuid
    ) -> error::Result<bool> {
        Ok(conn.execute(
            "delete from user_sessions where token = $1", 
            &[&token]
        ).await? == 1)
    }

    pub async fn insert(&self, conn: &impl GenericClient) -> error::Result<()> {
        conn.execute(
            "\
            insert into user_sessions \
            values ($1, $2, $3, $4, $5, $6)",
            &[
                &self.token,
                &self.owner,
                &self.dropped,
                &self.issued_on,
                &self.expires,
                &self.use_csrf
            ]
        ).await?;
        Ok(())
    }

    pub async fn delete(&self, conn: &impl GenericClient) -> error::Result<bool> {
        Self::delete_via_token(conn, &self.token).await
    }
}

pub async fn find_from_owner(
    conn: &impl GenericClient,
    owner: i32
) -> error::Result<Vec<UserSession>> {
    UserSession::find_from_owner(conn, owner).await
}

pub async fn find_from_token(
    conn: &impl GenericClient,
    token: uuid::Uuid
) -> error::Result<Option<UserSession>> {
    UserSession::find_from_token(conn, token).await
}

pub async fn delete(
    conn: &impl GenericClient,
    token: uuid::Uuid,
) -> error::Result<bool> {
    UserSession::delete_via_token(conn, &token).await
}