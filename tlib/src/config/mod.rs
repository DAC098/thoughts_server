use std::path::{PathBuf};

use lettre::address::{Address};

pub mod error;
pub mod shapes;

use shapes::MapShape;

// ----------------------------------------------------------------------------
// DBConfig
// ----------------------------------------------------------------------------

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

// ----------------------------------------------------------------------------
// SessionConfig
// ----------------------------------------------------------------------------

fn default_session_domain() -> String {
    "".to_owned()
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub domain: String
}

// ----------------------------------------------------------------------------
// EmailConfig
// ----------------------------------------------------------------------------

fn default_email_enable() -> bool {
    false
}

#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub enable: bool,
    pub from: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub relay: Option<String>
}

// ----------------------------------------------------------------------------
// ServerInfoConfig
// ----------------------------------------------------------------------------

fn default_info_secure() -> bool {
    false
}

fn default_info_origin() -> String {
    "".to_owned()
}

fn default_info_name() -> String {
    "Thoughts Server".to_owned()
}

#[derive(Debug, Clone)]
pub struct ServerInfoConfig {
    pub secure: bool,
    pub origin: String,
    pub name: String
}

// ----------------------------------------------------------------------------
// BindInterface
// ----------------------------------------------------------------------------

fn default_bind_port() -> u16 {
    8080
}

#[derive(Debug, Clone)]
pub struct BindInterface {
    pub host: String,
    pub port: u16
}

// ----------------------------------------------------------------------------
// TemplateConfig
// ----------------------------------------------------------------------------

fn default_template_directory() -> error::Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    dir.push("templates");

    Ok(dir)
}

fn default_template_dev_mode() -> bool {
    false
}

#[derive(Debug, Clone)]
pub struct TemplateConfig {
    pub directory: PathBuf,
    pub dev_mode: bool,
}

// ----------------------------------------------------------------------------
// FileServingConfig
// ----------------------------------------------------------------------------

fn default_file_serving_directory() -> error::Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    dir.push("static");

    Ok(dir)
}

#[derive(Debug, Clone)]
pub struct FileServingConfig {
    pub directory: PathBuf
}

// ----------------------------------------------------------------------------
// SslConfig
// ----------------------------------------------------------------------------

fn default_ssl_enable() -> bool {
    false
}

#[derive(Debug, Clone)]
pub struct SslConfig {
    pub enable: bool,
    pub key: Option<String>,
    pub cert: Option<String>
}

// ----------------------------------------------------------------------------
// ServerConfig
// ----------------------------------------------------------------------------

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
    pub ssl: SslConfig,
    pub template: TemplateConfig,
    pub file_serving: FileServingConfig,
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

fn load_file(config_file: std::path::PathBuf) -> error::Result<shapes::ServerConfigShape> {
    if let Some(ext) = config_file.extension() {
        if ext.eq("yaml") || ext.eq("yml") {
            Ok(serde_yaml::from_reader::<
                std::io::BufReader<std::fs::File>,
                shapes::ServerConfigShape
            >(std::io::BufReader::new(
                std::fs::File::open(&config_file)?
            ))?)
        } else if ext.eq("json") {
            Ok(serde_json::from_reader::<
                std::io::BufReader<std::fs::File>,
                shapes::ServerConfigShape
            >(std::io::BufReader::new(
                std::fs::File::open(&config_file)?
            ))?)
        } else {
            Err(error::Error::InvalidFileExtension(ext.to_os_string()))
        }
    } else {
        Err(error::Error::UnknownFileExtension)
    }
}

pub fn load_server_config(files: Vec<std::path::PathBuf>) -> error::Result<ServerConfig> {
    let mut base_shape = shapes::ServerConfigShape {
        bind: None, port: None,
        threads: None,
        backlog: None,
        max_connections: None,
        max_connection_rate: None,
        db: None,
        session: None,
        email: None,
        info: None,
        ssl: None,
        template: None,
        file_serving: None,
    };

    for file in files {
        base_shape.map_shape(load_file(file)?);
    }

    let mut bind_list: Vec<BindInterface>;
    let port = base_shape.port.unwrap_or(default_bind_port());

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
            domain: session.domain.unwrap_or(default_session_domain())
        }
    } else {
        SessionConfig {
            domain: default_session_domain()
        }
    };

    let email_config = if let Some(email) = base_shape.email {
        EmailConfig {
            enable: email.enable.unwrap_or(default_email_enable()),
            from: email.from,
            username: email.username,
            password: email.password,
            relay: email.relay
        }
    } else {
        EmailConfig {
            enable: default_email_enable(),
            from: None,
            username: None,
            password: None,
            relay: None
        }
    };

    let info_config = if let Some(info) = base_shape.info {
        ServerInfoConfig {
            secure: info.secure.unwrap_or(default_info_secure()),
            origin: info.origin.unwrap_or(default_info_origin()),
            name: info.name.unwrap_or(default_info_name())
        }
    } else {
        ServerInfoConfig {
            secure: default_info_secure(),
            origin: default_info_origin(),
            name: default_info_name()
        }
    };

    let ssl_config = if let Some(ssl) = base_shape.ssl {
        SslConfig {
            enable: ssl.enable.unwrap_or(default_ssl_enable()),
            key: ssl.key,
            cert: ssl.cert
        }
    } else {
        SslConfig {
            enable: default_ssl_enable(),
            key: None,
            cert: None
        }
    };

    let template_config = if let Some(template) = base_shape.template {
        TemplateConfig {
            directory: template.directory.unwrap_or(default_template_directory()?),
            dev_mode: template.dev_mode.unwrap_or(default_template_dev_mode()),
        }
    } else {
        TemplateConfig {
            directory: default_template_directory()?,
            dev_mode: default_template_dev_mode(),
        }
    };

    let file_serving_config = if let Some(file_serving) = base_shape.file_serving {
        FileServingConfig {
            directory: file_serving.directory.unwrap_or(default_file_serving_directory()?)
        }
    } else {
        FileServingConfig {
            directory: default_file_serving_directory()?
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
        ssl: ssl_config,
        template: template_config,
        file_serving: file_serving_config,
    })
}

pub fn validate_server_config(config: &ServerConfig) -> error::Result<()> {
    if config.bind.len() == 0 {
        return Err(error::Error::InvalidConfig(
            format!("no bind interfaces specified")
        ));
    }

    if config.email.enable {
        if config.email.username.is_none() || config.email.password.is_none() {
            return Err(error::Error::InvalidConfig(
                "username and password must be given if email is enabled".to_owned()
            ));
        }

        if config.email.from.is_none() {
            return Err(error::Error::InvalidConfig(
                "from email address must be given if email is enabled".to_owned()
            ));
        } else {
            if !config.email.from.as_ref().unwrap().parse::<Address>().is_ok() {
                return Err(error::Error::InvalidConfig("from email address is invalid".to_owned()));
            }
        }

        if config.email.relay.is_none() {
            return Err(error::Error::InvalidConfig(
                "relay must be given if email is emabled".to_owned()
            ));
        }
    }

    if !config.file_serving.directory.exists() {
        return Err(error::Error::InvalidConfig(
            "file_serving.directory does not exist".to_owned()
        ));
    } else if !config.file_serving.directory.is_dir() {
        return Err(error::Error::InvalidConfig(
            "file_serving.directory must be a directory".to_owned()
        ))
    }

    if !config.template.directory.exists() {
        return Err(error::Error::InvalidConfig(
            "template.directory does not exist".to_owned()
        ));
    } else if !config.template.directory.is_dir() {
        return Err(error::Error::InvalidConfig(
            "template.directory must be a directory".to_owned()
        ));
    }

    Ok(())
}