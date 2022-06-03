use std::{
    path::PathBuf, 
    convert::{TryFrom, TryInto}, 
    collections::HashMap, net::{SocketAddr, IpAddr}
};

use lettre::address::Address;
use shape_rs::MapShape;

pub mod error;
pub mod shapes;

// ----------------------------------------------------------------------------
// DBConfig
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DBConfig {
    pub username: String,
    pub password: String,
    pub database: String,
    pub hostname: String,
    pub port: u16
}

impl TryFrom<Option<shapes::DBConfigShape>> for DBConfig {
    type Error = error::Error;

    fn try_from(value: Option<shapes::DBConfigShape>) -> Result<Self, Self::Error> {
        if let Some(db) = value {
            Ok(DBConfig {
                username: db.username.unwrap_or("postgres".into()),
                password: db.password.unwrap_or("password".into()),
                database: db.database.unwrap_or("thoughts".into()),
                hostname: db.hostname.unwrap_or("localhost".into()),
                port: db.port.unwrap_or(5432)
            })
        } else {
            Ok(DBConfig {
                username: "postgres".into(),
                password: "password".into(),
                database: "thoughts".into(),
                hostname: "localhost".into(),
                port: 5432
            })
        }
    }
}

// ----------------------------------------------------------------------------
// SessionConfig
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub domain: String
}

impl TryFrom<Option<shapes::SessionConfigShape>> for SessionConfig {
    type Error = error::Error;

    fn try_from(value: Option<shapes::SessionConfigShape>) -> Result<Self, Self::Error> {
        if let Some(session) = value {
            Ok(SessionConfig {
                domain: session.domain.unwrap_or("".into())
            })
        } else {
            Ok(SessionConfig {
                domain: "".into()
            })
        }
    }
}

// ----------------------------------------------------------------------------
// EmailConfig
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub enable: bool,
    pub from: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub relay: Option<String>
}

impl TryFrom<Option<shapes::EmailConfigShape>> for EmailConfig {
    type Error = error::Error;

    fn try_from(value: Option<shapes::EmailConfigShape>) -> Result<Self, Self::Error> {
        if let Some(email) = value {
            Ok(EmailConfig {
                enable: email.enable.unwrap_or(false),
                from: email.from,
                username: email.username,
                password: email.password,
                relay: email.relay
            })
        } else {
            Ok(EmailConfig {
                enable: false,
                from: None,
                username: None,
                password: None,
                relay: None
            })
        }
    }
}

// ----------------------------------------------------------------------------
// ServerInfoConfig
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ServerInfoConfig {
    pub secure: bool,
    pub origin: String,
    pub name: String
}

impl TryFrom<Option<shapes::ServerInfoConfigShape>> for ServerInfoConfig {
    type Error = error::Error;

    fn try_from(value: Option<shapes::ServerInfoConfigShape>) -> Result<Self, Self::Error> {
        if let Some(info) = value {
            Ok(ServerInfoConfig {
                secure: info.secure.unwrap_or(false),
                origin: info.origin.unwrap_or("".into()),
                name: info.name.unwrap_or("Thoughts Server".into())
            })
        } else {
            Ok(ServerInfoConfig {
                secure: false,
                origin: "".into(),
                name: "Thoughts Server".into()
            })
        }
    }
}

// ----------------------------------------------------------------------------
// BindInterface
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BindInterfaceSsl {
    pub key: PathBuf,
    pub cert: PathBuf
}

#[derive(Debug, Clone)]
pub struct BindInterface {
    pub addr: SocketAddr,
    pub ssl: Option<BindInterfaceSsl>
}

// ----------------------------------------------------------------------------
// TemplateConfig
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TemplateConfig {
    pub directory: PathBuf,
    pub dev_mode: bool,
}

impl TryFrom<Option<shapes::TemplateConfigShape>> for TemplateConfig {
    type Error = error::Error;

    fn try_from(value: Option<shapes::TemplateConfigShape>) -> Result<Self, Self::Error> {
        let mut default_dir = std::env::current_dir()?;
        default_dir.push("templates");

        if let Some(template) = value {
            Ok(TemplateConfig {
                directory: template.directory.unwrap_or(default_dir),
                dev_mode: template.dev_mode.unwrap_or(false)
            })
        } else {
            Ok(TemplateConfig {
                directory: default_dir,
                dev_mode: false
            })
        }
    }
}

// ----------------------------------------------------------------------------
// FileServingConfig
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct FileServingConfig {
    pub directory: PathBuf,
    pub directories: HashMap<String, PathBuf>,
    pub files: HashMap<String, PathBuf>,
}

impl TryFrom<Option<shapes::FileServingConfigShape>> for FileServingConfig {
    type Error = error::Error;

    fn try_from(value: Option<shapes::FileServingConfigShape>) -> Result<Self, Self::Error> {
        let mut default_dir = std::env::current_dir()?;
        default_dir.push("static");

        if let Some(file_serving) = value {
            Ok(FileServingConfig {
                directory: file_serving.directory.unwrap_or(default_dir),
                directories: file_serving.directories.unwrap_or(HashMap::new()),
                files: file_serving.files.unwrap_or(HashMap::new())
            })
        } else {
            Ok(FileServingConfig {
                directory: default_dir,
                directories: HashMap::new(),
                files: HashMap::new()
            })
        }
    }
}

// ----------------------------------------------------------------------------
// StorageConfig
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub directory: PathBuf
}

impl TryFrom<Option<shapes::StorageConfigShape>> for StorageConfig {
    type Error = error::Error;

    fn try_from(value: Option<shapes::StorageConfigShape>) -> Result<Self, Self::Error> {
        let mut default_dir = std::env::current_dir()?;
        default_dir.push("storage");

        if let Some(storage) = value {
            Ok(StorageConfig {
                directory: storage.directory.unwrap_or(default_dir)
            })
        } else {
            Ok(StorageConfig {
                directory: default_dir
            })
        }
    }
}

// ----------------------------------------------------------------------------
// SslConfig
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SslConfig {
    pub key: Option<PathBuf>,
    pub cert: Option<PathBuf>
}

impl TryFrom<Option<shapes::SslConfigShape>> for SslConfig {
    type Error = error::Error;

    fn try_from(value: Option<shapes::SslConfigShape>) -> Result<Self, Self::Error> {
        if let Some(ssl) = value {
            Ok(SslConfig {
                key: ssl.key,
                cert: ssl.cert
            })
        } else {
            Ok(SslConfig {
                key: None,
                cert: None
            })
        }
    }
}

// ----------------------------------------------------------------------------
// ServerConfig
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind: HashMap<String, BindInterface>,

    pub threads: usize,
    pub backlog: u32,
    pub max_connections: usize,
    pub max_connection_rate: usize,

    pub db: DBConfig,
    pub session: SessionConfig,
    pub email: EmailConfig,
    pub info: ServerInfoConfig,
    pub template: TemplateConfig,
    pub file_serving: FileServingConfig,
    pub storage: StorageConfig,
}

impl TryFrom<shapes::ServerConfigShape> for ServerConfig {
    type Error = error::Error;

    fn try_from(value: shapes::ServerConfigShape) -> Result<Self, Self::Error> {
        let mut bind_list;
        let top_level_port = value.port.unwrap_or(8080);
        let top_level_ssl_key;
        let top_level_ssl_cert;

        if let Some(ssl) = value.ssl {
            top_level_ssl_key = ssl.key;
            top_level_ssl_cert = ssl.cert;
        } else {
            top_level_ssl_key = None;
            top_level_ssl_cert = None;
        }

        if let Some(bind) = value.bind {
            bind_list = HashMap::with_capacity(bind.len());

            for (bind_key, bind_value) in bind {
                if bind_value.is_none() {
                    continue;
                }

                let bind_value = bind_value.unwrap();
                let host: IpAddr;

                if let Some(h) = bind_value.host {
                    if let Ok(ip) = h.parse() {
                        host = ip;
                    } else {
                        let mut msg = String::from("invalid ip from host value. key: \"");
                        msg.push_str(&bind_key);
                        msg.push_str("\" given: \"");
                        msg.push_str(&h);
                        msg.push('"');

                        return Err(error::Error::InvalidConfig(msg))
                    }
                } else {
                    let mut msg = String::from("no ip value for host. key: \"");
                    msg.push_str(&bind_key);
                    msg.push('"');

                    return Err(error::Error::InvalidConfig(msg))
                }

                let sock_addr = SocketAddr::new(host, bind_value.port.unwrap_or(top_level_port));
                let mut ssl: Option<BindInterfaceSsl> = None;

                if let Some(bind_ssl) = bind_value.ssl {
                    match bind_ssl {
                        shapes::BindInterfaceSslShape::Bool(enable) => {
                            if enable {
                                if top_level_ssl_key.is_some() && top_level_ssl_cert.is_some() {
                                    ssl = Some(BindInterfaceSsl {
                                        key: top_level_ssl_key.clone().unwrap(),
                                        cert: top_level_ssl_cert.clone().unwrap()
                                    });
                                } else if top_level_ssl_key.is_some() {
                                    let msg = String::from("missing top level ssl cert");

                                    return Err(error::Error::InvalidConfig(msg));
                                } else {
                                    let msg = String::from("missing top level ssl key");

                                    return Err(error::Error::InvalidConfig(msg))
                                }
                            }
                        },
                        shapes::BindInterfaceSslShape::Struct {key: ssl_key_opt, cert: ssl_cert_opt} => {
                            let ssl_key;
                            let ssl_cert;

                            if let Some(unwrapped) = ssl_key_opt {
                                ssl_key = unwrapped
                            } else {
                                if let Some(unwrapped) = top_level_ssl_key.clone() {
                                    ssl_key = unwrapped;
                                } else {
                                    let mut msg = String::from("missing ssl key for bind interface \"");
                                    msg.push_str(&bind_key);
                                    msg.push('"');

                                    return Err(error::Error::InvalidConfig(msg));
                                }
                            }

                            if let Some(unwrapped) = ssl_cert_opt {
                                ssl_cert = unwrapped;
                            } else {
                                if let Some(unwrapped) = top_level_ssl_cert.clone() {
                                    ssl_cert = unwrapped;
                                } else {
                                    let mut msg = String::from("missing ssl cert for bind interface \"");
                                    msg.push_str(&bind_key);
                                    msg.push('"');

                                    return Err(error::Error::InvalidConfig(msg));
                                }
                            }

                            ssl = Some(BindInterfaceSsl {
                                key: ssl_key,
                                cert: ssl_cert
                            });
                        }
                    }
                }

                bind_list.insert(bind_key, BindInterface {
                    addr: sock_addr,
                    ssl
                });
            }
        } else {
            bind_list = HashMap::new();
        }

        Ok(ServerConfig {
            bind: bind_list,

            threads: value.threads.unwrap_or(num_cpus::get()),
            backlog: value.backlog.unwrap_or(2048),
            max_connections: value.max_connections.unwrap_or(25000),
            max_connection_rate: value.max_connection_rate.unwrap_or(256),

            db: value.db.try_into()?,
            session: value.session.try_into()?,
            email: value.email.try_into()?,
            info: value.info.try_into()?,
            template: value.template.try_into()?,
            file_serving: value.file_serving.try_into()?,
            storage: value.storage.try_into()?
        })
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

pub fn load_server_config(files: Vec<std::path::PathBuf>) -> error::Result<ServerConfig> {
    let mut base_shape: shapes::ServerConfigShape = std::default::Default::default();

    for file in files {
        let shape = shapes::ServerConfigShape::try_from(&file)?;
        let parent = file.parent().unwrap();

        base_shape.map_shape(shapes::validate_server_config_shape(&parent, shape)?);
    }

    let conf = base_shape.try_into()?;

    validate_server_config(&conf)?;

    Ok(conf)
}

pub fn validate_server_config(config: &ServerConfig) -> error::Result<()> {
    if config.bind.len() == 0 {
        let msg = String::from("no bind interface specified");

        return Err(error::Error::InvalidConfig(msg));
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

    Ok(())
}