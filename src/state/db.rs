use actix_web::web;
use bb8_postgres::{PostgresConnectionManager, bb8::Pool, bb8::PooledConnection};
use tokio_postgres::NoTls;

use crate::net::http::error;

pub struct DBState {
    pub pool: Pool<PostgresConnectionManager<NoTls>>
}

pub type WebDbState = web::Data<DBState>;

impl DBState {
    pub async fn get_conn(&self) -> error::Result<PooledConnection<'_, PostgresConnectionManager<NoTls>>> {
        self.pool.get().await.map_err(
            |e| error::ResponseError::BB8Error(e)
        )
    }
}

impl From<Pool<PostgresConnectionManager<NoTls>>> for DBState {
    fn from(pool: Pool<PostgresConnectionManager<NoTls>>) -> Self {
        Self { pool }
    }
}