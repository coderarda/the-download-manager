use std::sync::Mutex;

use crate::download::DownloadStatus;

pub struct TauriState {
    pub downloads: Mutex<Vec<DownloadStatus>>,
}