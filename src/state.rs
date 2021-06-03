use bb8_postgres::{PostgresConnectionManager, bb8::Pool, bb8::PooledConnection};
use tokio_postgres::{NoTls};
use lettre::message::{Mailbox};
use lettre::transport::smtp::{SmtpTransport};
use lettre::transport::smtp::authentication::{Credentials};

use crate::error;
use crate::config;
use crate::email;

#[derive(Clone)]
pub struct ServerInfoState {
    pub secure: bool,
    pub origin: String,
    pub name: String
}

impl ServerInfoState {

    pub fn new(conf: config::ServerInfoConfig) -> Self {
        Self {
            secure: conf.secure,
            origin: conf.origin,
            name: conf.name
        }
    }
    
    pub fn url_origin(&self) -> String {
        format!(
            "{}://{}",
            if self.secure { "https" } else { "http" },
            self.origin
        )
    }
}

#[derive(Clone)]
pub struct EmailState {
    pub enabled: bool,
    pub credentials: Option<Credentials>,
    pub relay: Option<String>,
    pub from: Option<Mailbox>
}

impl EmailState {

    pub fn new(conf: config::EmailConfig) -> EmailState {
        let credentials = if conf.enable {
            Some(email::get_credentials(
                conf.username.unwrap(),
                conf.password.unwrap()
            ))
        } else { None };
        let from = if conf.enable {
            Some(email::get_mailbox(
                conf.from.unwrap(),
                None
            ).unwrap())
        } else { None };
        let relay = if conf.enable {
            conf.relay
        } else {
            None
        };

        EmailState {
            enabled: conf.enable,
            credentials,
            relay,
            from
        }
    }
    
    pub fn get_transport(&self) -> error::Result<SmtpTransport> {
        Ok(email::get_smtp_transport(
            self.relay.as_ref().expect("email_relay has not been set"),
            self.credentials.as_ref().expect("email_credentials has not been set").clone()
        )?)
    }

    pub fn get_from(&self) -> Mailbox {
        self.from.as_ref().expect("email_from has not been set").clone()
    }
}

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<PostgresConnectionManager<NoTls>>,

    pub email: EmailState,
    pub info: ServerInfoState
}

impl AppState {

    pub fn new(
        pool: Pool<PostgresConnectionManager<NoTls>>, 
        email_config: config::EmailConfig,
        info_config: config::ServerInfoConfig
    ) -> AppState {
        AppState {
            pool: pool.clone(),
            email: EmailState::new(email_config),
            info: ServerInfoState::new(info_config)
        }
    }

    pub async fn get_conn(&self) -> error::Result<PooledConnection<'_, PostgresConnectionManager<NoTls>>> {
        self.pool.get().await.map_err(
            |e| error::ResponseError::BB8Error(e)
        )
    }
}