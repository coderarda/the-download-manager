use std::{
    fs::File,
    io::Write,
    sync::{Arc, Mutex}, time::Duration,
};

use curl::easy::Handler;
use tauri::{AppHandle, Manager};

use super::DownloadStatus;

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
        while self.download.lock().unwrap().paused {
            std::thread::sleep(Duration::from_millis(100));
        }
        self.download.lock().unwrap().set_curr_size(dlnow as u64);
        self.handle.trigger_global("onbackendupdate", Some(dlnow.to_string()));
        if dlnow == dltotal && dlnow as u64 != 0 {
            self.download.lock().unwrap().set_finished();
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
