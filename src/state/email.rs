use actix_web::web;
use lettre::message::Mailbox;
use lettre::transport::smtp::SmtpTransport;
use lettre::transport::smtp::authentication::Credentials;

use crate::config::EmailConfig;
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
        let from = if conf.enable {
            let given = conf.from.unwrap()
                .parse()
                .unwrap();

            Some(Mailbox::new(None, given))
        } else { None };
        
        let credentials = if conf.enable {
            Some(Credentials::new(
                conf.username.unwrap(),
                conf.password.unwrap()
            ))
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

    pub fn has_credentials(&self) -> bool {
        self.credentials.is_some()
    }

    #[allow(dead_code)]
    pub fn get_credentials(&self) -> Credentials {
        self.credentials.clone().expect("credentials has not been set")
    }

    pub fn has_relay(&self) -> bool {
        self.relay.is_some()
    }

    // pub fn get_relay(&self) -> String {
    //     self.relay.clone().expect("relay has not been set")
    // }

    pub fn has_from(&self) -> bool {
        self.from.is_some()
    }

    pub fn get_from(&self) -> Mailbox {
        self.from.clone().expect("from has not been set")
    }

    // pub fn can_get_transport(&self) -> bool {
    //     self.relay.is_some() && self.credentials.is_some()
    // }

    pub fn get_transport(&self) -> error::Result<SmtpTransport> {
        let relay = self.relay.as_ref()
            .expect("email_relay has not been set");
        let credentials = self.credentials.as_ref()
            .expect("email_credentials has not been set")
            .clone();

        let rtn = SmtpTransport::relay(relay)?
            .credentials(credentials)
            .build();

        Ok(rtn)
    }
}