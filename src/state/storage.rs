use std::path::PathBuf;
use std::ffi::OsStr;
use std::fs;

use actix_web::web;

use crate::config::StorageConfig;

use crate::error;

pub struct StorageState {
    dir: PathBuf,
    tmp: PathBuf
}

pub type WebStorageState = web::Data<StorageState>;

impl StorageState {

    fn check_create_dir(name: &str, path: &PathBuf) -> error::Result<()> {
        if path.exists() {
            if !path.is_dir() {
                return Err(error::AppError::General(
                    format!("storage {} directory must be a directory: {}", name, path.display())
                ));
            }
        } else {
            fs::create_dir(&path)?;
        }

        Ok(())
    }

    pub fn new(conf: StorageConfig) -> error::Result<StorageState> {
        StorageState::check_create_dir("data", &conf.directory)?;
        StorageState::check_create_dir("tmp", &conf.temp)?;

        Ok(StorageState {
            dir: conf.directory,
            tmp: conf.temp
        })
    }

    pub fn get_tmp_dir_ref(&self) -> &PathBuf {
        &self.tmp
    }

    pub fn get_audio_file_path<E>(&self, user_id: &i32, entry_id: &i32, audio_id: &i32, extension: E) -> PathBuf
    where
        E: AsRef<OsStr>
     {
        let mut audio_file = self.dir.clone();
        audio_file.push("users");
        audio_file.push(user_id.to_string());
        audio_file.push("entries");
        audio_file.push(entry_id.to_string());
        audio_file.push("audio");
        audio_file.set_file_name(audio_id.to_string());
        audio_file.set_extension(extension);

        audio_file
    }
}