use std::sync::Arc;
use tokio::sync::Mutex;

use super::download::DownloadStatus;

#[derive(Clone)]
pub struct AppDownloadManager {
    downloads: Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>>,
}

impl AppDownloadManager {
    pub fn new(downloads: Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>>) -> Self {
        AppDownloadManager { downloads }
    }

    pub fn get_downloads(&self) -> Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>> {
        self.downloads.clone()
    }
}