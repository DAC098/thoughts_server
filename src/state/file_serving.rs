use std::{collections::HashMap, path::PathBuf, time::SystemTime, sync::atomic::{AtomicU64, Ordering}, borrow::Borrow, hash::Hash};

use actix_web::web;
use tokio::sync::RwLock;

use crate::config;

struct CacheData {
    path: PathBuf,
    access: AtomicU64
}

pub struct FileServingState {
    pub directories: HashMap<String, PathBuf>,
    pub files: HashMap<String, PathBuf>,
    cache: tokio::sync::RwLock<HashMap<String, CacheData>>
}

pub type WebFileServingState = web::Data<FileServingState>;

impl FileServingState {
    fn get_now() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub async fn cache_file<K>(&self, path: K, file: PathBuf) -> () 
    where
        K: Into<String>
    {
        let mut write_lock = self.cache.write().await;
        write_lock.insert(path.into(), CacheData {
            path: file,
            access: AtomicU64::new(Self::get_now())
        });
    }

    pub async fn check_cache<K>(&self, path: &K) -> Option<PathBuf> 
    where
        K: ?Sized + Hash + Eq,
        String: Borrow<K>
    {
        let read = self.cache.read().await;

        if let Some(data) = read.get(path) {
            data.access.store(Self::get_now(), Ordering::Release);
            Some(data.path.clone())
        } else {
            None
        }
    }
}

impl From<config::FileServingConfig> for FileServingState {
    fn from(conf: config::FileServingConfig) -> Self {
        FileServingState {
            directories: conf.directories,
            files: conf.files,
            cache: RwLock::new(HashMap::new())
        }
    }
}