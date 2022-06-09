use actix_web::web;

use crate::config;

pub struct SessionState {
    domain: String
}

impl SessionState {
    pub fn get_domain(&self) -> &String {
        &self.domain
    }
}

impl From<config::SessionConfig> for SessionState {
    fn from(conf: config::SessionConfig) -> Self {
        Self { domain: conf.domain }
    }
}

pub struct SecurityState {
    session: SessionState,
    secret: String
}

pub type WebSecurityState = web::Data<SecurityState>;

impl SecurityState {
    pub fn get_session(&self) -> &SessionState {
        &self.session
    }

    pub fn get_secret(&self) -> &String {
        &self.secret
    }
}

impl From<config::SecurityConfig> for SecurityState {
    fn from(conf: config::SecurityConfig) -> Self {
        Self { session: conf.session.into(), secret: conf.secret }
    }
}