use std::{collections::HashMap, path::PathBuf};

use actix_web::web;

use crate::config;

pub struct FileServingState {
    pub directories: HashMap<String, PathBuf>,
    pub files: HashMap<String, PathBuf>
}

pub type WebFileServingState = web::Data<FileServingState>;

impl From<config::FileServingConfig> for FileServingState {
    fn from(conf: config::FileServingConfig) -> Self {
        FileServingState {
            directories: conf.directories,
            files: conf.files 
        }
    }
}