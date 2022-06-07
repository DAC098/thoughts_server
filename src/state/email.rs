use actix_web::web;
use lettre::message::Mailbox;
use lettre::transport::smtp::SmtpTransport;
use lettre::transport::smtp::authentication::Credentials;

use crate::config::EmailConfig;
use crate::email;
use crate::response::error;

pub struct EmailState {
    pub enabled: bool,
    pub credentials: Option<Credentials>,
    pub relay: Option<String>,
    pub from: Option<Mailbox>
}

pub type WebEmailState = web::Data<EmailState>;

impl EmailState {

    pub fn new(conf: EmailConfig) -> EmailState {
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

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[allow(dead_code)]
    pub fn get_relay(&self) -> Option<String> {
        self.relay.as_ref().map(|r| r.clone())
    }

    pub fn can_get_transport(&self) -> bool {
        self.relay.is_some() && self.credentials.is_some()
    }
    
    pub fn get_transport(&self) -> error::Result<SmtpTransport> {
        Ok(email::get_smtp_transport(
            self.relay.as_ref().expect("email_relay has not been set"),
            self.credentials.as_ref().expect("email_credentials has not been set").clone()
        )?)
    }

    pub fn has_from(&self) -> bool {
        self.from.is_some()
    }

    pub fn get_from(&self) -> Option<Mailbox> {
        self.from.as_ref().map(|f| f.clone())
    }
}