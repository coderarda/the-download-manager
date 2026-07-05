use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::download::DownloadStatus;

#[derive(Clone)]
pub struct AppDownloadManager {
    downloads: Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>>,
    next_id: Arc<AtomicU32>,
}

impl AppDownloadManager {
    pub fn new(downloads: Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>>) -> Self {
        AppDownloadManager {
            downloads,
            next_id: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn get_downloads(&self) -> Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>> {
        self.downloads.clone()
    }

    pub fn get_next_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn set_next_id(&self, id: u32) {
        self.next_id.store(id, Ordering::SeqCst);
    }
}
