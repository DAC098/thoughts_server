use std::path::{PathBuf};
use std::ffi::{OsStr};
use std::fs::{self};

use tlib::config::{StorageConfig};

use crate::error;

pub struct StorageState {
    pub audio: PathBuf
}

impl StorageState {

    pub fn new(conf: StorageConfig) -> error::Result<StorageState> {
        let mut audio = conf.directory.clone();
        audio.push("audio");

        if audio.exists() {
            if !audio.is_dir() {
                return Err(error::AppError::General(
                    format!("storage audio directory must be a directory: {}", audio.display())
                ));
            }
        } else {
            fs::create_dir(&audio)?;
        }
        
        Ok(StorageState { audio })
    }
    
    pub fn get_audio_file_path<E>(&self, user_id: &i32, entry_id: &i32, audio_id: &i32, extension: E) -> PathBuf
    where
        E: AsRef<OsStr>
     {
        let mut audio_file = self.audio.clone();
        audio_file.push(user_id.to_string());
        audio_file.push(entry_id.to_string());
        audio_file.set_file_name(audio_id.to_string());
        audio_file.set_extension(extension);

        audio_file
    }
}