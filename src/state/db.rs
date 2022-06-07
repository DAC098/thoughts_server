use actix_web::web;
use bb8_postgres::{PostgresConnectionManager, bb8::Pool, bb8::PooledConnection};
use tokio_postgres::NoTls;

use crate::response::error;

pub struct DBState {
    pool: Pool<PostgresConnectionManager<NoTls>>
}

pub type WebDbState = web::Data<DBState>;

impl DBState {

    pub fn new(
        pool: Pool<PostgresConnectionManager<NoTls>>
    ) -> DBState {
        DBState { pool }
    }

    pub fn get_pool(&self) -> &Pool<PostgresConnectionManager<NoTls>> {
        &self.pool
    }

    pub async fn get_conn(&self) -> error::Result<PooledConnection<'_, PostgresConnectionManager<NoTls>>> {
        self.pool.get().await.map_err(
            |e| error::ResponseError::BB8Error(e)
        )
    }
}