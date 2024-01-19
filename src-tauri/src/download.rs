use std::{io::Write, path::Path, time::Duration};
use serde::{Deserialize, Serialize};
use tauri::Manager;

extern crate reqwest;

#[derive(Clone, Serialize, Deserialize)]
pub struct DownloadObj {
    id: u32,
    url: String,
    title: String,
    filesize: u64,
}

#[derive(Clone)]
pub struct DownloadStatus {
    pub item: DownloadObj,
    paused: bool,
    current_range_ptr: u64,
    resume: bool,
}

impl DownloadStatus {
    pub fn new(item: DownloadObj) -> Self {
        Self {
            item,
            paused: false,
            current_range_ptr: 0,
            resume: false,
        }
    }

    pub fn set_pause(&mut self) {
        self.paused = true;
    }

    pub async fn pause_if_requested(&mut self, curr_ptr: u64) {
        if self.paused && !self.resume {
            while !self.resume {
                tokio::time::sleep(Duration::from_millis(10)).await;
                self.current_range_ptr = curr_ptr as u64;
            }
        }
    }

    pub fn resume_download(&mut self) {
        if !self.resume {
            self.resume = true;
        }
    }

    pub async fn download_self(mut self, handle: tauri::AppHandle) {
        // Add request pausing via range header and convert single request into reqwest client
        let mut file = std::fs::File::create(Path::new(
            tauri::api::path::download_dir()
                .unwrap()
                .join(self.item.get_file_name())
                .as_path(),
        ))
        .unwrap();
        let client = reqwest::Client::new();
        let range_header_value = format!("bytes={}-", self.current_range_ptr);
        let mut resp = client
            .get(self.item.get_url())
            .header(reqwest::header::RANGE, range_header_value)
            .send()
            .await
            .expect("Error while downloading!");
        let mut range_ptr: u64 = 0;
        while let Some(buf) = resp.chunk().await.expect("No chunk found!") {
            self.pause_if_requested(range_ptr).await;
            file.write(&buf)
                .expect("Could not write to file!");
            let len = buf.len();
            let update = serde_json::to_string(&DownloadInfo::new(
                self.item.get_id(),
                len as u64
            ))
            .unwrap();
            range_ptr += buf.len() as u64;
            handle.emit_all("ondownloadupdate", update).unwrap();
        }
    }
}

impl DownloadObj {
    pub fn get_url(&self) -> &String {
        &self.url
    }

    pub fn get_file_name(&self) -> &String {
        &self.title
    }

    pub fn get_id(&self) -> u32 {
        self.id
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
