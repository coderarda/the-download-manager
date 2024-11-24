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
    pub fn new(id: u8, url: String, title: String, filesize: u64) -> Self {
        DownloadObj {
            id,
            url,
            title,
            filesize
        }
    }

    pub fn get_file_name(&self) -> &String {
        &self.title
    }

    pub fn concat_number(&mut self) {
        self.title.push_str("(1)");
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
    finished: bool,
    downloading: bool,
    curr_size: u64,
}

impl DownloadStatus {
    pub fn new(item: DownloadObj) -> Self {
        Self {
            item,
            paused: false,
            finished: false,
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

    pub fn set_finished(&mut self) {
        self.finished = true;
        self.paused = false;
        self.downloading = false;
    }

    pub fn is_finished(&self) -> bool {
        self.finished
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

    pub fn set_item(&mut self, item: DownloadObj) {
        self.item = item
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
