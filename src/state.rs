use bb8_postgres::{PostgresConnectionManager, bb8::Pool, bb8::PooledConnection};
use tokio_postgres::{NoTls};

use crate::error;

pub struct AppState {
    pub pool: Pool<PostgresConnectionManager<NoTls>>
}



impl AppState {
    
    pub fn new(pool: &Pool<PostgresConnectionManager<NoTls>>) -> AppState {
        AppState { pool: pool.clone() }
    }

    pub async fn get_conn(&self) -> error::Result<PooledConnection<'_, PostgresConnectionManager<NoTls>>> {
        self.pool.get().await.map_err(
            |e| error::ResponseError::BB8Error(e)
        )
    }
}