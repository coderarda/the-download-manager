use std::sync::Arc;
use tokio::sync::Mutex;

use super::download::DownloadStatus;

pub struct TauriState {
    pub downloads: Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>>,
}