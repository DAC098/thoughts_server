use serde::{Deserialize, Serialize};
use serde_json::{error::Result};

fn default_db_username() -> String {
    "postgres".to_owned()
}

fn default_db_password() -> String {
    "password".to_owned()
}

fn default_db_database() -> String {
    "thoughts".to_owned()
}

fn default_db_port() -> u16 {
    5432
}

fn default_db_hostname() -> String {
    "localhost".to_owned()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DBConfig {
    #[serde(default = "default_db_username")]
    pub username: String,
    #[serde(default = "default_db_password")]
    pub password: String,

    #[serde(default = "default_db_database")]
    pub database: String,

    #[serde(default = "default_db_hostname")]
    pub hostname: String,

    #[serde(default = "default_db_port")]
    pub port: u16
}

impl Default for DBConfig {
    fn default() -> DBConfig {
        DBConfig {
            username: default_db_username(),
            password: default_db_password(),
            database: default_db_database(),

            hostname: default_db_hostname(),
            port: default_db_port()
        }
    }
}

fn default_hostname() -> Vec<String> {
    vec!["0.0.0.0".to_owned(), "::1".to_owned()]
}

fn default_port() -> u16 {
    8080
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_hostname")]
    pub host: Vec<String>,
    #[serde(default = "default_port")]
    pub port: u16,

    pub db: DBConfig,

    pub session_domain: Option<String>,

    pub key: Option<String>,
    pub cert: Option<String>
}

impl Default for ServerConfig {
    fn default() -> ServerConfig {
        ServerConfig {
            host: default_hostname(),
            port: default_port(),

            db: Default::default(),

            session_domain: None,

            key: None,
            cert: None
        }
    }
}

pub fn load_server_config(file: String) -> Result<ServerConfig> {
    let config_file = std::fs::File::open(file);
    let config: ServerConfig = Default::default();

    if config_file.is_ok() {
        serde_json::from_reader::<
            std::io::BufReader<std::fs::File>,
            ServerConfig
        >(std::io::BufReader::new(config_file.unwrap()))
    } else {
        Ok(config)
    }
}