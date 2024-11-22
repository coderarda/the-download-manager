use std::{cmp::Ordering, fs::File, sync::Arc, io::Write};
use reqwest::header::RANGE;
use tauri::AppHandle;
use tokio::sync::Mutex;
use tauri::Manager;
use futures_util::StreamExt;

use crate::util::download::DownloadInfo;

use super::download::DownloadStatus;

#[derive(Clone)]
pub struct AppDownloadManager {
    downloads: Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>>,
}

impl AppDownloadManager {
    pub fn new(downloads: Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>>) -> Self {
        AppDownloadManager {
            downloads
        }
    }

    pub fn get_downloads(&self) -> Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>> {
        self.downloads.clone()
    }

    // Replace old function with this one.
    pub async fn download_with_pause(self, status: Arc<Mutex<DownloadStatus>>, handle: AppHandle) -> Result<(), reqwest::Error> {
        println!("Download starting...");
        let h = handle.clone();
        let id = status.lock().await.get_item().get_id();
        status.lock().await.set_downloading();
        let arg1 = status.lock().await.get_curr_size();
        let size = status.lock().await.get_item().get_total_size();
        let client = reqwest::Client::new();
        let res = client
            .get(status.lock().await.get_item().get_url())
            .header(RANGE, format!("bytes={arg1}-{size}"))
            .send()
            .await?;
    
        // Check if the server accepts range requests
        tokio::spawn(async move {
            let mut file: Option<File>;
            let mut dir = tauri::api::path::download_dir()
                .unwrap()
                .join(status.lock().await.get_item().get_file_name());
            if res
                .headers()
                .get("accept-ranges")
                .map_or(false, |v| v == "bytes")
            {
                match arg1.cmp(&size) {
                    Ordering::Less => {
                        if status.lock().await.get_curr_size() == 0 {
                            file = Some(std::fs::File::create(dir.as_path()).unwrap());
                        } else {
                            file = Some(
                                std::fs::OpenOptions::new()
                                    .append(true)
                                    .open(dir.as_path())
                                    .unwrap(),
                            );
                        }
                    }
                    Ordering::Equal => {
                        status.lock().await.get_item().concat_number();
                        dir.pop();
                        dir.push(status.lock().await.get_item().get_file_name());
                        file = Some(std::fs::File::create(dir.as_path()).unwrap());
                    }
                    Ordering::Greater => {
                        panic!("Anomaly occurred! Canceling...");
                        // Send event to frontend to delete file and try again
                    }
                }
    
                let mut stream = res.bytes_stream();
                let mut new_size = status.lock().await.get_curr_size();
                while let Some(b) = stream.next().await {
                    if status.lock().await.is_paused() {
                        return Err::<(), u32>(status.lock().await.get_item().get_id());
                    }
                    match b {
                        Ok(chunk) => {
                            new_size += chunk.len() as u64;
                            let update =
                                serde_json::to_string(&DownloadInfo::new(id, new_size)).unwrap();
                            println!("{}", update);
                            h.emit_all("ondownloadupdate", update).unwrap();
                            match file {
                                Some(ref mut f) => {
                                    f.write_all(&chunk).unwrap();
                                    status.lock().await.set_curr_size(new_size);
                                }
                                None => {
                                    println!("File not open!");
                                }
                            }
                        }
                        Err(e) => {
                            println!("Error reading stream: {:?}", e);
                            return Err::<(), u32>(status.lock().await.get_item().get_id());
                        }
                    }
                }
            }
            let curr = status.lock().await.get_curr_size();
            if curr == size {
                status.lock().await.set_finished();
                if self.get_downloads().lock().await.len() != 0 {
                    self.get_downloads()
                        .lock()
                        .await
                        .remove(status.lock().await.get_item().get_id() as usize);
                }
            }
            Ok(())
        })
        .await
        .unwrap()
        .unwrap_or_else(|e| {
            println!("Download with id {e} has been paused.");
        });
    
        Ok(())
    }

    pub fn download_without_pause(&self, status: DownloadStatus, handle: AppHandle) {
        
    }
}