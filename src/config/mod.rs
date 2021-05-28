use serde::{Deserialize};

pub mod error;

trait MapShape {
    fn map_shape(&mut self, rhs: Self);
}

#[derive(Deserialize)]
pub struct DBConfigShape {
    pub username: Option<String>,
    pub password: Option<String>,

    pub database: Option<String>,

    pub hostname: Option<String>,
    pub port: Option<u16>
}

impl MapShape for DBConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        if rhs.username.is_some() {
            self.username = rhs.username;
        }
    
        if rhs.password.is_some() {
            self.password = rhs.password;
        }
    
        if rhs.database.is_some() {
            self.database = rhs.database;
        }
    
        if rhs.hostname.is_some() {
            self.hostname = rhs.hostname;
        }
    
        if rhs.port.is_some() {
            self.port = rhs.port;
        }
    }
}

#[derive(Deserialize)]
pub struct BindInterfaceShape {
    pub host: String,
    pub port: Option<u16>
}

#[derive(Deserialize)]
pub struct SessionConfigShape {
    pub domain: Option<String>
}

impl MapShape for SessionConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        if rhs.domain.is_some() {
            self.domain = rhs.domain;
        }
    }
}

#[derive(Deserialize)]
pub struct EmailConfigShape {
    pub enable: Option<bool>,
    pub from: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub relay: Option<String>
}

impl MapShape for EmailConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        if rhs.enable.is_some() {
            self.enable = rhs.enable;
        }
    
        if rhs.from.is_some() {
            self.from = rhs.from;
        }
    
        if rhs.username.is_some() {
            self.username = rhs.username;
        }
    
        if rhs.password.is_some() {
            self.password = rhs.password;
        }
    
        if rhs.relay.is_some() {
            self.relay = rhs.relay;
        }
    }
}

#[derive(Deserialize)]
pub struct ServerInfoConfigShape {
    pub secure: Option<bool>,
    pub origin: Option<String>,
    pub name: Option<String>
}

impl MapShape for ServerInfoConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        if rhs.secure.is_some() {
            self.secure = rhs.secure;
        }

        if rhs.origin.is_some() {
            self.origin = rhs.origin;
        }

        if rhs.name.is_some() {
            self.name = rhs.name;
        }
    }
}

#[derive(Deserialize)]
pub struct ServerConfigShape {
    pub bind: Option<Vec<BindInterfaceShape>>,
    pub port: Option<u16>,

    pub threads: Option<usize>,
    pub backlog: Option<u32>,
    pub max_connections: Option<usize>,
    pub max_connection_rate: Option<usize>,

    pub db: Option<DBConfigShape>,
    pub session: Option<SessionConfigShape>,
    pub email: Option<EmailConfigShape>,
    pub info: Option<ServerInfoConfigShape>,

    pub key: Option<String>,
    pub cert: Option<String>
}

impl MapShape for ServerConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        if rhs.bind.is_some() {
            self.bind = rhs.bind;
        }
    
        if rhs.port.is_some() {
            self.port = rhs.port;
        }
    
        if rhs.threads.is_some() {
            self.threads = rhs.threads;
        }
    
        if rhs.backlog.is_some() {
            self.backlog = rhs.backlog;
        }
    
        if rhs.max_connections.is_some() {
            self.max_connections = rhs.max_connections;
        }
    
        if rhs.max_connection_rate.is_some() {
            self.max_connection_rate = rhs.max_connection_rate;
        }
    
        if rhs.key.is_some() {
            self.key = rhs.key;
        }
    
        if rhs.cert.is_some() {
            self.cert = rhs.cert;
        }
    
        if let Some(session) = self.session.as_mut() {
            if let Some(rhs_session) = rhs.session {
                session.map_shape(rhs_session);
            }
        } else {
            self.session = rhs.session;
        }
    
        if let Some(db) = self.db.as_mut() {
            if let Some(rhs_db) = rhs.db {
                db.map_shape(rhs_db);
            }
        } else {
            self.db = rhs.db;
        }
    
        if let Some(email) = self.email.as_mut() {
            if let Some(rhs_email) = rhs.email {
                email.map_shape(rhs_email);
            }
        } else {
            self.email = rhs.email;
        }

        if let Some(info) = self.info.as_mut() {
            if let Some(rhs_info) = rhs.info {
                info.map_shape(rhs_info);
            }
        } else {
            self.info = rhs.info;
        }
    }
}

fn load_file(config_file: std::path::PathBuf) -> error::Result<ServerConfigShape> {
    if let Some(ext) = config_file.extension() {
        if ext.eq("yaml") || ext.eq("yml") {
            Ok(serde_yaml::from_reader::<
                std::io::BufReader<std::fs::File>,
                ServerConfigShape
            >(std::io::BufReader::new(
                std::fs::File::open(&config_file)?
            ))?)
        } else if ext.eq("json") {
            Ok(serde_json::from_reader::<
                std::io::BufReader<std::fs::File>,
                ServerConfigShape
            >(std::io::BufReader::new(
                std::fs::File::open(&config_file)?
            ))?)
        } else {
            Err(error::ConfigError::InvalidFileExtension(ext.to_os_string()))
        }
    } else {
        Err(error::ConfigError::UnknownFileExtension)
    }
}

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

#[derive(Debug, Clone)]
pub struct DBConfig {
    pub username: String,
    pub password: String,

    pub database: String,

    pub hostname: String,
    pub port: u16
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub domain: String
}

#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub enable: bool,
    pub from: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub relay: Option<String>
}

#[derive(Debug, Clone)]
pub struct ServerInfoConfig {
    pub secure: bool,
    pub origin: String,
    pub name: String
}

#[derive(Debug, Clone)]
pub struct BindInterface {
    pub host: String,
    pub port: u16
}

fn default_bind() -> Vec<BindInterface> {
    vec![
        BindInterface {
            host: "0.0.0.0".to_owned(),
            port: 8080
        },
        BindInterface {
            host: "::1".to_owned(),
            port: 8080
        }
    ]
}

fn default_backlog() -> u32 {
    2048
}

fn default_max_connections() -> usize {
    25000
}

fn default_max_connection_rate() -> usize {
    256
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind: Vec<BindInterface>,

    pub threads: usize,
    pub backlog: u32,
    pub max_connections: usize,
    pub max_connection_rate: usize,

    pub db: DBConfig,

    pub session: SessionConfig,

    pub email: EmailConfig,

    pub info: ServerInfoConfig,

    pub key: Option<String>,
    pub cert: Option<String>
}

pub fn load_server_config(files: Vec<std::path::PathBuf>) -> error::Result<ServerConfig> {
    let mut base_shape = ServerConfigShape {
        bind: None, port: None,
        threads: None,
        backlog: None,
        max_connections: None,
        max_connection_rate: None,
        db: None,
        session: None,
        email: None,
        info: None,
        key: None, cert: None
    };

    for file in files {
        base_shape.map_shape(load_file(file)?);
    }

    let mut bind_list: Vec<BindInterface>;
    let port = base_shape.port.unwrap_or(8080);

    if let Some(bind) = base_shape.bind {
        bind_list = Vec::with_capacity(bind.len());

        for interface in bind {
            bind_list.push(BindInterface {
                host: interface.host,
                port: interface.port.unwrap_or(port)
            });
        }
    } else {
        bind_list = default_bind();
    }

    // this looks a little wired but works for what is
    // needed. equivalent to a ternary operator in other
    // languages
    let db_config = if let Some(db) = base_shape.db {
        DBConfig {
            hostname: db.hostname.unwrap_or(default_db_hostname()),
            username: db.username.unwrap_or(default_db_username()),
            password: db.password.unwrap_or(default_db_password()),
            port: db.port.unwrap_or(default_db_port()),
            database: db.database.unwrap_or(default_db_database()),
        }
    } else {
        DBConfig {
            hostname: default_db_hostname(),
            username: default_db_username(),
            password: default_db_password(),
            port: default_db_port(),
            database: default_db_database()
        }
    };

    let session_config = if let Some(session) = base_shape.session {
        SessionConfig {
            domain: session.domain.unwrap_or("".to_owned())
        }
    } else {
        SessionConfig {
            domain: "".to_owned()
        }
    };

    let email_config = if let Some(email) = base_shape.email {
        EmailConfig {
            enable: email.enable.unwrap_or(false),
            from: email.from,
            username: email.username,
            password: email.password,
            relay: email.relay
        }
    } else {
        EmailConfig {
            enable: false,
            from: None,
            username: None,
            password: None,
            relay: None
        }
    };

    let info_config = if let Some(info) = base_shape.info {
        ServerInfoConfig {
            secure: info.secure.unwrap_or(false),
            origin: info.origin.unwrap_or("".to_owned()),
            name: info.name.unwrap_or("Thoughts Server".to_owned())
        }
    } else {
        ServerInfoConfig {
            secure: false,
            origin: "".to_owned(),
            name: "Thoughts Server".to_owned()
        }
    };

    Ok(ServerConfig {
        bind: bind_list,
        threads: base_shape.threads.unwrap_or(num_cpus::get()),
        backlog: base_shape.backlog.unwrap_or(default_backlog()),
        max_connections: base_shape.max_connections.unwrap_or(default_max_connections()),
        max_connection_rate: base_shape.max_connection_rate.unwrap_or(default_max_connection_rate()),
        db: db_config,
        session: session_config,
        email: email_config,
        info: info_config,
        key: base_shape.key,
        cert: base_shape.cert
    })
}