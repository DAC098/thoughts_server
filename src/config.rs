use serde::{Deserialize, Serialize};
use serde_json::{error::Result};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DBConfig {
    pub username: String,
    pub password: String,

    pub hostname: String,
    pub port: u16
}

impl Default for DBConfig {
    fn default() -> DBConfig {
        DBConfig {
            username: String::from("postgres"),
            password: String::from("password"),

            hostname: String::from("localhost"),
            port: 5432
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub host: Vec<String>,
    pub port: u16,

    pub db: DBConfig,

    pub session_domain: Option<String>,

    pub key: Option<String>,
    pub cert: Option<String>
}

impl Default for ServerConfig {
    fn default() -> ServerConfig {
        ServerConfig {
            host: vec![String::from("0.0.0.0"), String::from("::1")],
            port: 8080,

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
        let reader = std::io::BufReader::new(config_file.unwrap());
        return serde_json::from_reader::<
            std::io::BufReader<std::fs::File>,
            ServerConfig
        >(reader);
    }

    return Ok(config);
}