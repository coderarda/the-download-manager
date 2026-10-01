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

#[cfg(test)]
mod tests {
    use super::AppDownloadManager;
    use crate::util::download::{DownloadObj, DownloadStatus};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn build_status(id: u32) -> DownloadStatus {
        DownloadStatus::new(
            DownloadObj::new(
                id,
                "https://example.com/file.bin".to_string(),
                format!("file-{}.bin", id),
                100,
            ),
            false,
            false,
            0,
        )
    }

    #[tokio::test]
    async fn returns_same_downloads_collection() {
        let base = Arc::new(Mutex::new(vec![Arc::new(Mutex::new(build_status(1)))]));
        let manager = AppDownloadManager::new(base.clone());

        let downloaded = manager.get_downloads();
        let list = downloaded.lock().await;

        assert_eq!(list.len(), 1);
        let first = list[0].lock().await;
        assert_eq!(first.get_item().get_id(), 1);
    }

    #[test]
    fn next_id_increments_and_respects_setter() {
        let manager = AppDownloadManager::new(Arc::new(Mutex::new(Vec::new())));

        assert_eq!(manager.get_next_id(), 0);
        assert_eq!(manager.get_next_id(), 1);

        manager.set_next_id(10);

        assert_eq!(manager.get_next_id(), 10);
        assert_eq!(manager.get_next_id(), 11);
    }
}
