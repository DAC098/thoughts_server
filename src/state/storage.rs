use std::path::PathBuf;
use std::ffi::OsStr;
use std::fs;

use crate::config::StorageConfig;

use crate::error;

pub struct StorageState {
    audio: PathBuf,
    tmp: PathBuf
}

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
        let mut audio = conf.directory.clone();
        audio.push("audio");
        let mut tmp = conf.directory.clone();
        tmp.push("tmp");

        StorageState::check_create_dir("audio", &audio)?;
        StorageState::check_create_dir("tmp", &tmp)?;
        
        Ok(StorageState { audio, tmp })
    }

    // pub fn get_audio_dir(&self) -> PathBuf {
    //     self.audio.clone()
    // }

    // pub fn get_tmp_dir(&self) -> PathBuf {
    //     self.tmp.clone()
    // }

    pub fn get_tmp_dir_ref(&self) -> &PathBuf {
        &self.tmp
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