use serde::{Deserialize};

pub mod error;

trait MapShape {
    fn map_shape(&mut self, rhs: Self);
}

#[inline]
fn assign_map_value<T>(lhs: &mut Option<T>, rhs: Option<T>) {
    if rhs.is_some() { *lhs = rhs; }
}

#[inline]
fn assign_map_struct<T>(lhs: &mut Option<T>, rhs: Option<T>) 
where
    T: MapShape
{
    if let Some(lhs_value) = lhs.as_mut() {
        if let Some(rhs_value) = rhs {
            lhs_value.map_shape(rhs_value);
        }
    } else {
        *lhs = rhs;
    }
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
        assign_map_value(&mut self.username, rhs.username);
        assign_map_value(&mut self.password, rhs.password);
        assign_map_value(&mut self.database, rhs.database);
        assign_map_value(&mut self.hostname, rhs.hostname);
        assign_map_value(&mut self.port, rhs.port);
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
        assign_map_value(&mut self.domain, rhs.domain);
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
        assign_map_value(&mut self.enable, rhs.enable);
        assign_map_value(&mut self.from, rhs.from);
        assign_map_value(&mut self.username, rhs.username);
        assign_map_value(&mut self.password, rhs.password);
        assign_map_value(&mut self.relay, rhs.relay);
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
        assign_map_value(&mut self.secure, rhs.secure);
        assign_map_value(&mut self.origin, rhs.origin);
        assign_map_value(&mut self.name, rhs.name);
    }
}

#[derive(Deserialize)]
pub struct SslConfigShape {
    pub enable: Option<bool>,
    pub key: Option<String>,
    pub cert: Option<String>
}

impl MapShape for SslConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        assign_map_value(&mut self.enable, rhs.enable);
        assign_map_value(&mut self.key, rhs.key);
        assign_map_value(&mut self.cert, rhs.cert);
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
    pub ssl: Option<SslConfigShape>
}

impl MapShape for ServerConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        assign_map_value(&mut self.bind, rhs.bind);
        assign_map_value(&mut self.port, rhs.port);
        assign_map_value(&mut self.threads, rhs.threads);
        assign_map_value(&mut self.backlog, rhs.backlog);
        assign_map_value(&mut self.max_connections, rhs.max_connections);
        assign_map_value(&mut self.max_connection_rate, rhs.max_connection_rate);
    
        assign_map_struct(&mut self.session, rhs.session);
        assign_map_struct(&mut self.db, rhs.db);
        assign_map_struct(&mut self.email, rhs.email);
        assign_map_struct(&mut self.info, rhs.info);
        assign_map_struct(&mut self.ssl, rhs.ssl);
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
pub struct SslConfig {
    pub enable: bool,
    pub key: Option<String>,
    pub cert: Option<String>
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
    pub ssl: SslConfig
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
        ssl: None
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

    let ssl_config = if let Some(ssl) = base_shape.ssl {
        SslConfig {
            enable: ssl.enable.unwrap_or(false),
            key: ssl.key,
            cert: ssl.cert
        }
    } else {
        SslConfig {
            enable: false,
            key: None,
            cert: None
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
        ssl: ssl_config
    })
}