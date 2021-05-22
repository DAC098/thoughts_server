use serde::{Deserialize};

pub mod error;

#[derive(Deserialize)]
pub struct DBConfigShape {
    pub username: Option<String>,
    pub password: Option<String>,

    pub database: Option<String>,

    pub hostname: Option<String>,
    pub port: Option<u16>
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

    pub key: Option<String>,
    pub cert: Option<String>
}

fn map_db_config_shape(lhs: &mut DBConfigShape, rhs: DBConfigShape) {
    if rhs.username.is_some() {
        lhs.username = rhs.username;
    }

    if rhs.password.is_some() {
        lhs.password = rhs.password;
    }

    if rhs.database.is_some() {
        lhs.database = rhs.database;
    }

    if rhs.hostname.is_some() {
        lhs.hostname = rhs.hostname;
    }

    if rhs.port.is_some() {
        lhs.port = rhs.port;
    }
}

fn map_session_config_shape(lhs: &mut SessionConfigShape, rhs: SessionConfigShape) {
    if rhs.domain.is_some() {
        lhs.domain = rhs.domain;
    }
}

fn map_server_config_shape(lhs: &mut ServerConfigShape, rhs: ServerConfigShape) {
    if rhs.bind.is_some() {
        lhs.bind = rhs.bind;
    }

    if rhs.port.is_some() {
        lhs.port = rhs.port;
    }

    if rhs.threads.is_some() {
        lhs.threads = rhs.threads;
    }

    if rhs.backlog.is_some() {
        lhs.backlog = rhs.backlog;
    }

    if rhs.max_connections.is_some() {
        lhs.max_connections = rhs.max_connections;
    }

    if rhs.max_connection_rate.is_some() {
        lhs.max_connection_rate = rhs.max_connection_rate;
    }

    if rhs.key.is_some() {
        lhs.key = rhs.key;
    }

    if rhs.cert.is_some() {
        lhs.cert = rhs.cert;
    }

    if let Some(session) = lhs.session.as_mut() {
        if rhs.session.is_some() {
            map_session_config_shape(session, rhs.session.unwrap());
        }
    } else {
        lhs.session = rhs.session;
    }

    if let Some(db) = lhs.db.as_mut() {
        if let Some(rhs_db) = rhs.db {
            map_db_config_shape(db, rhs_db);
        }
    } else {
        lhs.db = rhs.db;
    }
}

fn load_file(config_file: &std::path::Path) -> error::Result<ServerConfigShape> {
    if config_file.exists() && config_file.is_file() {
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
    } else {
        Err(error::ConfigError::NotReadableFile(config_file.as_os_str().to_os_string()))
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

    pub key: Option<String>,
    pub cert: Option<String>
}

pub fn load_server_config(files: Vec<&std::path::Path>) -> error::Result<ServerConfig> {
    let mut base_shape = ServerConfigShape {
        bind: None, port: None,
        threads: None,
        backlog: None,
        max_connections: None,
        max_connection_rate: None,
        db: None,
        session: None,
        key: None, cert: None
    };

    for file in files {
        map_server_config_shape(&mut base_shape, load_file(&file)?);
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

    Ok(ServerConfig {
        bind: bind_list,
        threads: base_shape.threads.unwrap_or(num_cpus::get()),
        backlog: base_shape.backlog.unwrap_or(default_backlog()),
        max_connections: base_shape.max_connections.unwrap_or(default_max_connections()),
        max_connection_rate: base_shape.max_connection_rate.unwrap_or(default_max_connection_rate()),
        db: db_config,
        session: session_config,
        key: base_shape.key,
        cert: base_shape.cert
    })
}