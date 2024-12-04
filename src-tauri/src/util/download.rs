use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DownloadObj {
    id: u8,
    url: String,
    title: String,
    filesize: u64,
}

impl DownloadObj {

    pub fn get_file_name(&self) -> &String {
        &self.title
    }

    pub fn get_id(&self) -> u8 {
        self.id
    }

    pub fn get_url(&self) -> String {
        self.url.clone()
    }

    pub fn get_total_size(&self) -> u64{
        self.filesize
    }
}

#[derive(Clone, Debug)]
pub struct DownloadStatus {
    item: DownloadObj,
    paused: bool,
    downloading: bool,
    curr_size: u64,
}

impl DownloadStatus {
    pub fn new(item: DownloadObj) -> Self {
        Self {
            item,
            paused: false,
            downloading: false,
            curr_size: 0,
        }
    }

    pub fn set_curr_size(&mut self, size: u64) {
        self.curr_size = size;
    }
    
    pub fn set_pause(&mut self) {
        self.paused = true;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn set_downloading(&mut self) {
        self.downloading = true;
        self.paused = false;
    }

    pub fn get_item(&self) -> DownloadObj {
        self.item.clone()
    }

    pub fn get_curr_size(&self) -> u64 {
        self.curr_size
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
