use std::fmt::Display;

use serde::{Deserialize, Serialize};
use tokio::sync::MutexGuard;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct DownloadObj {
    id: u8,
    url: String,
    title: String,
    filesize: u64,
}

impl DownloadObj {
    pub fn new(id: u8, url: String, title: String, filesize: u64) -> Self {
        Self {
            id,
            url,
            title,
            filesize,
        }
    }

    pub fn get_file_name(&self) -> &String {
        &self.title
    }

    pub fn get_id(&self) -> u8 {
        self.id
    }

    pub fn get_url(&self) -> String {
        self.url.clone()
    }

    /*
    pub fn get_total_size(&self) -> u64{
        self.filesize
    }
    */
    
    pub fn set_id(&mut self, id: u8) {
        self.id = id;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DownloadStatus {
    item: DownloadObj,
    paused: bool,
    downloading: bool,
}

impl DownloadStatus {
    pub fn new(item: DownloadObj, paused: bool, downloading: bool) -> Self {
        Self {
            item,
            paused,
            downloading,
        }
    }

    pub fn from_mutex_guard(guard: &MutexGuard<'_, DownloadStatus>) -> Self {
        Self { item: guard.get_item(), paused: guard.paused.clone() , downloading: guard.is_downloading() }
    }

    pub fn set_pause(&mut self) {
        self.paused = true;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn is_downloading(&self) -> bool {
        self.downloading
    }

    pub fn set_downloading(&mut self) {
        self.downloading = true;
        self.paused = false;
    }

    pub fn get_item(&self) -> DownloadObj {
        self.item.clone()
    }
}

impl From<String> for DownloadStatus {
    fn from(item: String) -> Self {
        serde_json::from_str(&item).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DownloadInfo {
    id: u8,
    chunk_size: u64,
}

impl DownloadInfo {
    pub fn new(id: u8, chunk_size: u64) -> Self {
        Self { id, chunk_size }
    }
}

impl Display for DownloadInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id: {}, chunk_size: {}", self.id, self.chunk_size)
    }
}
