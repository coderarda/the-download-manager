use std::{sync::Mutex, collections::VecDeque};

use crate::download::DownloadStatus;

pub struct TauriState {
    pub downloads: Mutex<VecDeque<DownloadStatus>>,
}