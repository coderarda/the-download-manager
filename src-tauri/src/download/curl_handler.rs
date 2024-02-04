use std::{fs::File, io::Write, sync::{Arc, Mutex}};

use curl::easy::Handler;
use tauri::{AppHandle, Manager};

use super::{DownloadInfo, DownloadStatus};

pub struct MyCurlHandler {
    download: Arc<Mutex<DownloadStatus>>,
    file: File,
    handle: AppHandle,
}

impl MyCurlHandler {
    pub fn new(download: Arc<Mutex<DownloadStatus>>, file: File, handle: AppHandle) -> Self {
        Self {
            download,
            file,
            handle,
        }
    }
}

impl Handler for MyCurlHandler {
    fn progress(&mut self, dltotal: f64, dlnow: f64, _ultotal: f64, _ulnow: f64) -> bool {
        if !self.download.lock().unwrap().paused {
            let update =
                serde_json::to_string(&DownloadInfo::new(self.download.lock().unwrap().item.id, dlnow as u64))
                    .unwrap();
            self.handle
                .emit_all("ondownloadupdate", update)
                .expect("Failed to send progress!");
            if dlnow == dltotal && dlnow as u64 != 0 {
                self.download.lock().unwrap().set_finished();
            }
        }
        true
    }

    fn write(&mut self, data: &[u8]) -> Result<usize, curl::easy::WriteError> {
        if self.download.lock().unwrap().paused {
            Err(curl::easy::WriteError::Pause)
        } else {
            self.file.write_all(data).unwrap();
            Ok(data.len())
        }
    }
}
