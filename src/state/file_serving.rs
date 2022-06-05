use std::{collections::HashMap, path::PathBuf};

use crate::config;

pub struct FileServingState {
    pub directories: HashMap<String, PathBuf>,
    pub files: HashMap<String, PathBuf>
}

impl From<config::FileServingConfig> for FileServingState {
    fn from(conf: config::FileServingConfig) -> Self {
        FileServingState {
            directories: conf.directories,
            files: conf.files 
        }
    }
}