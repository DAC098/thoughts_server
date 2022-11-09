use actix_web::web;

use crate::{config, security};

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
    secret: String,
    signing: security::mac::Algorithm
}

pub type WebSecurityState = web::Data<SecurityState>;

impl SecurityState {
    pub fn get_session(&self) -> &SessionState {
        &self.session
    }

    pub fn get_secret(&self) -> &String {
        &self.secret
    }

    pub fn get_signing(&self) -> &security::mac::Algorithm {
        &self.signing
    }
}

impl From<config::SecurityConfig> for SecurityState {
    fn from(conf: config::SecurityConfig) -> Self {
        let signing = match conf.signing_algo.as_str() {
            "blake3" => security::mac::Algorithm::BLAKE3,
            "sha224" => security::mac::Algorithm::HMAC_SHA224,
            "sha256" => security::mac::Algorithm::HMAC_SHA256,
            "sha384" => security::mac::Algorithm::HMAC_SHA384,
            "sha512" => security::mac::Algorithm::HMAC_SHA512,
            _ => panic!("unknown security.signing_algo config value \"{}\"", conf.signing_algo)
        };

        Self { 
            session: conf.session.into(), 
            secret: conf.secret,
            signing
        }
    }
}