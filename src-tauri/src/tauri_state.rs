use std::sync::{Arc, Mutex};

use crate::download::DownloadStatus;

pub struct TauriState {
    pub downloads: Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>>,
}