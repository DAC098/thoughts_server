use std::collections::HashMap;
use std::fs::canonicalize;
use std::{path::{Path, PathBuf}, convert::TryFrom};
use std::default::Default;
use std::io::ErrorKind as IoErrorKind;

use serde::Deserialize;
use shape_rs::{MapShape, assign_map_struct};

use super::error;

#[derive(Debug, Deserialize)]
pub struct DBConfigShape {
    pub username: Option<String>,
    pub password: Option<String>,

    pub database: Option<String>,

    pub hostname: Option<String>,
    pub port: Option<u16>
}

impl MapShape for DBConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        self.username.map_shape(rhs.username);
        self.password.map_shape(rhs.password);
        self.database.map_shape(rhs.database);
        self.hostname.map_shape(rhs.hostname);
        self.port.map_shape(rhs.port);
    }
}

#[derive(Debug, Deserialize)]
pub struct BindInterfaceShape {
    pub host: String,
    pub port: Option<u16>
}

#[derive(Debug, Deserialize)]
pub struct SessionConfigShape {
    pub domain: Option<String>
}

impl MapShape for SessionConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        self.domain.map_shape(rhs.domain);
    }
}

#[derive(Debug, Deserialize)]
pub struct EmailConfigShape {
    pub enable: Option<bool>,
    pub from: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub relay: Option<String>
}

impl MapShape for EmailConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        self.enable.map_shape(rhs.enable);
        self.from.map_shape(rhs.from);
        self.username.map_shape(rhs.username);
        self.password.map_shape(rhs.password);
        self.relay.map_shape(rhs.relay);
    }
}

#[derive(Debug, Deserialize)]
pub struct ServerInfoConfigShape {
    pub secure: Option<bool>,
    pub origin: Option<String>,
    pub name: Option<String>
}

impl MapShape for ServerInfoConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        self.secure.map_shape(rhs.secure);
        self.origin.map_shape(rhs.origin);
        self.name.map_shape(rhs.name);
    }
}

#[derive(Debug, Deserialize)]
pub struct SslConfigShape {
    pub enable: Option<bool>,
    pub key: Option<PathBuf>,
    pub cert: Option<PathBuf>
}

impl MapShape for SslConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        self.enable.map_shape(rhs.enable);
        self.key.map_shape(rhs.key);
        self.cert.map_shape(rhs.cert);
    }
}

#[derive(Debug, Deserialize)]
pub struct TemplateConfigShape {
    pub directory: Option<PathBuf>,
    pub dev_mode: Option<bool>,
}

impl MapShape for TemplateConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        self.directory.map_shape(rhs.directory);
        self.dev_mode.map_shape(rhs.dev_mode);
    }
}

#[derive(Debug, Deserialize)]
pub struct FileServingConfigShape {
    pub directory: Option<PathBuf>,
    pub directories: Option<HashMap<String, PathBuf>>,
    pub files: Option<HashMap<String,PathBuf>>,
}

impl MapShape for FileServingConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        if let Some(map) = self.directories.as_mut() {
            if let Some(rhs_map) = rhs.directories {
                for (key, path) in rhs_map {
                    map.insert(key, path);
                }
            }
        } else if let Some(rhs_map) = rhs.directories {
            self.directories = Some(rhs_map);
        }

        if let Some(map) = self.files.as_mut() {
            if let Some(rhs_map) = rhs.files {
                for (key, path) in rhs_map {
                    map.insert(key, path);
                }
            }
        } else if let Some(rhs_map) = rhs.files {
            self.files = Some(rhs_map)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct StorageConfigShape {
    pub directory: Option<PathBuf>
}

impl MapShape for StorageConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        self.directory.map_shape(rhs.directory);
    }
}

#[derive(Debug, Deserialize)]
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
    pub ssl: Option<SslConfigShape>,
    pub template: Option<TemplateConfigShape>,
    pub file_serving: Option<FileServingConfigShape>,
    pub storage: Option<StorageConfigShape>,
}

impl MapShape for ServerConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        self.bind.map_shape(rhs.bind);
        self.port.map_shape(rhs.port);
        self.threads.map_shape(rhs.threads);
        self.backlog.map_shape(rhs.backlog);
        self.max_connections.map_shape(rhs.max_connections);
        self.max_connection_rate.map_shape(rhs.max_connection_rate);
    
        assign_map_struct(&mut self.session, rhs.session);
        assign_map_struct(&mut self.db, rhs.db);
        assign_map_struct(&mut self.email, rhs.email);
        assign_map_struct(&mut self.info, rhs.info);
        assign_map_struct(&mut self.ssl, rhs.ssl);
        assign_map_struct(&mut self.template, rhs.template);
        assign_map_struct(&mut self.file_serving, rhs.file_serving);
        assign_map_struct(&mut self.storage, rhs.storage);
    }
}

impl Default for ServerConfigShape {
    fn default() -> ServerConfigShape {
        ServerConfigShape {
            bind: None,
            port: None,
            
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
            storage: None,
        }
    }
}

impl TryFrom<&PathBuf> for ServerConfigShape {
    type Error = error::Error;

    fn try_from(config_file: &PathBuf) -> Result<Self, Self::Error> {
        if let Some(ext) = config_file.extension() {
            let ext = ext.to_ascii_lowercase();
            let reader = std::io::BufReader::new(std::fs::File::open(config_file)?);

            if ext.eq("yaml") || ext.eq("yml") {
                Ok(serde_yaml::from_reader(reader)?)
            } else if ext.eq("json") {
                Ok(serde_json::from_reader(reader)?)
            } else {
                Err(error::Error::InvalidFileExtension(ext.to_os_string()))
            }
        } else {
            Err(error::Error::UnknownFileExtension)
        }
    }
}

fn validate_path_buf(conf_dir: &Path, name: &str, is_dir: bool, directory: PathBuf) -> error::Result<PathBuf> {
    let to_canonicalize = if directory.has_root() {
        directory
    } else {
        let mut with_root = conf_dir.clone().to_owned();
        with_root.push(directory);
        with_root
    };

    match canonicalize(&to_canonicalize) {
        Ok(path) => {
            if is_dir {
                if !path.is_dir() {
                    Err(error::Error::InvalidConfig(
                        format!(
                            "requested {} is not a directory.\nconfig file: {}\ngiven value: {}\nreal path: {}",
                            name,
                            conf_dir.display(), 
                            to_canonicalize.display(),
                            path.display()
                        )
                    ))
                } else {
                    Ok(path)
                }
            } else {
                if !path.is_file() {
                    Err(error::Error::InvalidConfig(
                        format!(
                            "requested {} is not a file.\nconfig file: {}\ngiven value: {}\nreal path: {}",
                            name,
                            conf_dir.display(),
                            to_canonicalize.display(),
                            path.display()
                        )
                    ))
                } else {
                    Ok(path)
                }
            }
        },
        Err(error) => match error.kind() {
            IoErrorKind::NotFound => Err(error::Error::InvalidConfig(
                format!(
                    "requested {} directory was not found.\nconfig file: {}\ngive value: {}",
                    name,
                    conf_dir.display(), 
                    to_canonicalize.display()
                )
            )),
            _ => Err(error.into())
        }
    }
}

pub fn validate_server_config_shape(conf_dir: &Path, mut conf: ServerConfigShape) -> error::Result<ServerConfigShape> {
    conf.file_serving = if let Some(mut file_serving) = conf.file_serving {
        file_serving.directories = if let Some(directories) = file_serving.directories {
            let mut verified_map = HashMap::with_capacity(directories.len());

            for (mut key, value) in directories {
                if !key.ends_with('/') {
                    key.push('/');
                }

                let mut name = String::from("static directory map (conf.file_serving.directories.\"");
                name.reserve(key.len() + 2);
                name.push_str(&key);
                name.push_str("\")");

                verified_map.insert(key, validate_path_buf(conf_dir, &name, true, value)?);
            }

            Some(verified_map)
        } else {
            None
        };

        file_serving.files = if let Some(files) = file_serving.files {
            let mut verified_map = HashMap::with_capacity(files.len());

            for (key, value) in files {
                let mut name = String::from("static file map (conf.file_serving.files.\"");
                name.reserve(key.len() + 2);
                name.push_str(&key);
                name.push_str("\")");

                verified_map.insert(key, validate_path_buf(conf_dir, &name, false, value)?);
            }

            Some(verified_map)
        } else {
            None
        };

        Some(file_serving)
    } else {
        None
    };

    conf.template = if let Some(mut template) = conf.template {
        template.directory = if let Some(template_directory) = template.directory {
            Some(validate_path_buf(conf_dir, "config template directory (conf.template.directory)", true, template_directory)?)
        } else {
            None
        };

        Some(template)
    } else {
        None
    };

    conf.storage = if let Some(mut storage) = conf.storage {
        storage.directory = if let Some(storage_directory) = storage.directory {
            Some(validate_path_buf(conf_dir, "config storage directory (conf.storage.directory)", true, storage_directory)?)
        } else {
            None
        };

        Some(storage)
    } else {
        None
    };

    conf.ssl = if let Some(mut ssl) = conf.ssl {
        ssl.cert = if let Some(ssl_cert) = ssl.cert {
            Some(validate_path_buf(conf_dir, "config ssl certificate file (conf.ssl.cert)", false, ssl_cert)?)
        } else {
            None
        };

        ssl.key = if let Some(ssl_key) = ssl.key {
            Some(validate_path_buf(conf_dir, "config ssl key file (conf.ssl.key)", false, ssl_key)?)
        } else {
            None
        };

        Some(ssl)
    } else {
        None
    };

    Ok(conf)
}