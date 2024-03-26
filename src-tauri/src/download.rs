use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DownloadObj {
    id: u32,
    url: String,
    title: String,
    filesize: u64,
}

#[derive(Clone, Debug)]
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

    pub fn get_size(&self) -> u64 {
        self.curr_size
    }
}

impl DownloadObj {
    pub fn get_file_name(&self) -> &String {
        &self.title
    }

    pub fn concat_number(&mut self) {
        self.title.push_str("(1)");
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_url(&self) -> String {
        self.url.clone()
    }

    pub fn get_total_size(&self) -> u64{
        self.filesize
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
