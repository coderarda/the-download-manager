use serde::{Deserialize, Serialize};


pub mod curl_handler;

#[derive(Clone, Serialize, Deserialize)]
pub struct DownloadObj {
    id: u32,
    url: String,
    title: String,
    filesize: u64,
}

#[derive(Clone)]
pub struct DownloadStatus {
    item: DownloadObj,
    paused: bool,
    resume: bool,
    finished: bool,
    downloading: bool,
    curr_size: u64,
}

impl DownloadStatus {
    pub fn new(item: DownloadObj) -> Self {
        Self {
            item,
            paused: false,
            resume: false,
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
        self.resume = false;
    }

    pub fn set_finished(&mut self) {
        self.finished = true;
        self.resume = false; 
        self.paused = false;
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn set_downloading(&mut self) {
        self.downloading = true;
    }

    pub fn is_downloading(&self) -> bool {
        self.downloading
    }

    pub fn resume_download(&mut self) {
        if !self.resume {
            self.paused = false;
            self.resume = true;
        }
    }

    pub fn will_resume(&self) -> bool {
        self.resume
    }

    pub fn set_resume_false(&mut self) {
        self.resume = false;
    }

    pub fn get_item(&self) -> DownloadObj {
        self.item.clone()
    }
}

impl DownloadObj {
    pub fn get_file_name(&self) -> &String {
        &self.title
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_url(&self) -> String {
        self.url.clone()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DownloadInfo {
    id: u32,
    chunk_size: u64,
}

impl DownloadInfo {
    pub fn new(id: u32, chunk_size: u64) -> Self {
        Self { id, chunk_size }
    }
}
