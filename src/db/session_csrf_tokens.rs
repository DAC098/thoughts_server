use tokio_postgres::GenericClient;

use crate::db::error;

#[allow(dead_code)]
pub struct SessionCsrfToken {
    pub token: String,
    pub session_token: uuid::Uuid,
    pub issued_on: chrono::DateTime<chrono::Utc>,
    pub expires: chrono::DateTime<chrono::Utc>
}

impl SessionCsrfToken {

    #[allow(dead_code)]
    pub async fn find_from_token(conn: &impl GenericClient, token: &String) -> error::Result<Option<SessionCsrfToken>> {
        if let Some(record) = conn.query_opt(
            "\
            select session_token, \
                   issued_on, \
                   expires \
            from session_csrf_tokens \
            where token = $1",
            &[token]
        ).await? {
            Ok(Some(SessionCsrfToken {
                token: token.clone(),
                session_token: record.get(0),
                issued_on: record.get(1),
                expires: record.get(2)
            }))
        } else {
            Ok(None)
        }
    }
}