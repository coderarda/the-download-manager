use std::sync::Arc;
use tokio::sync::Mutex;

use crate::download::DownloadStatus;

pub struct TauriState {
    pub downloads: Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>>,
}