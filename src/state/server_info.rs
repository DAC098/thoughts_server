use actix_web::web;

use crate::config::ServerInfoConfig;

pub struct ServerInfoState {
    pub secure: bool,
    pub origin: String,
    pub name: String
}

pub type WebServerInfoState = web::Data<ServerInfoState>;

impl ServerInfoState {

    pub fn new(conf: ServerInfoConfig) -> ServerInfoState {
        ServerInfoState {
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