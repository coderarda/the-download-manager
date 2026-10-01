use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tokio::sync::MutexGuard;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct DownloadObj {
    id: u32,
    url: String,
    title: String,
    filesize: u64,
}

impl DownloadObj {
    pub fn new(id: u32, url: String, title: String, filesize: u64) -> Self {
        Self {
            id,
            url,
            title,
            filesize,
        }
    }

    pub fn get_file_name(&self) -> &String {
        &self.title
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_url(&self) -> String {
        self.url.clone()
    }

    pub fn get_total_size(&self) -> u64 {
        self.filesize
    }
    pub fn set_id(&mut self, id: u32) {
        self.id = id;
    }
    pub fn set_file_name(&mut self, name: String) {
        self.title = name;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DownloadStatus {
    item: DownloadObj,
    paused: bool,
    downloading: bool,
    current_sz: u64,
    #[serde(default)]
    pausable: bool,
}

impl DownloadStatus {
    pub fn new(item: DownloadObj, paused: bool, downloading: bool, current_sz: u64) -> Self {
        Self {
            item,
            paused,
            downloading,
            current_sz,
            pausable: false,
        }
    }

    pub fn from_mutex_guard(guard: &MutexGuard<'_, DownloadStatus>) -> Self {
        Self {
            item: guard.get_item(),
            paused: guard.paused.clone(),
            downloading: guard.is_downloading(),
            current_sz: guard.get_current_sz(),
            pausable: guard.is_pausable(),
        }
    }

    pub fn set_pause(&mut self) {
        self.paused = true;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn is_downloading(&self) -> bool {
        self.downloading
    }

    pub fn set_downloading(&mut self) {
        self.downloading = true;
        self.paused = false;
    }

    pub fn get_item(&self) -> DownloadObj {
        self.item.clone()
    }

    pub fn get_current_sz(&self) -> u64 {
        self.current_sz
    }

    pub fn set_current_sz(&mut self, sz: u64) {
        self.current_sz = sz;
    }

    pub fn is_pausable(&self) -> bool {
        self.pausable
    }

    pub fn set_pausable(&mut self, pausable: bool) {
        self.pausable = pausable;
    }

    pub fn set_item_id(&mut self, id: u32) {
        self.item.set_id(id);
    }
    pub fn set_item_file_name(&mut self, name: String) {
        self.item.set_file_name(name);
    }
}

impl From<String> for DownloadStatus {
    fn from(item: String) -> Self {
        serde_json::from_str(&item).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DownloadInfo {
    id: u32,
    chunk_size: u64,
}

impl DownloadInfo {
    pub fn new(id: u32, chunk_size: u64) -> Self {
        Self { id, chunk_size }
    }
}

impl Display for DownloadInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id: {}, chunk_size: {}", self.id, self.chunk_size)
    }
}

#[cfg(test)]
mod tests {
    use super::{DownloadInfo, DownloadObj, DownloadStatus};
    use tokio::sync::Mutex;

    fn sample_obj() -> DownloadObj {
        DownloadObj::new(
            11,
            "https://example.com/archive.zip".to_string(),
            "archive.zip".to_string(),
            2048,
        )
    }

    #[test]
    fn download_obj_getters_and_setters_work() {
        let mut obj = sample_obj();

        assert_eq!(obj.get_id(), 11);
        assert_eq!(obj.get_file_name(), "archive.zip");
        assert_eq!(obj.get_url(), "https://example.com/archive.zip".to_string());
        assert_eq!(obj.get_total_size(), 2048);

        obj.set_id(99);
        obj.set_file_name("renamed.zip".to_string());

        assert_eq!(obj.get_id(), 99);
        assert_eq!(obj.get_file_name(), "renamed.zip");
    }

    #[test]
    fn download_status_state_transitions_work() {
        let obj = sample_obj();
        let mut status = DownloadStatus::new(obj, false, false, 12);

        assert!(!status.is_paused());
        assert!(!status.is_downloading());
        assert!(!status.is_pausable());

        status.set_pause();
        assert!(status.is_paused());

        status.set_downloading();
        assert!(status.is_downloading());
        assert!(!status.is_paused());

        status.set_current_sz(33);
        status.set_pausable(true);
        assert_eq!(status.get_current_sz(), 33);
        assert!(status.is_pausable());

        status.set_item_id(77);
        status.set_item_file_name("final.zip".to_string());
        let item = status.get_item();
        assert_eq!(item.get_id(), 77);
        assert_eq!(item.get_file_name(), "final.zip");
    }

    #[tokio::test]
    async fn download_status_can_be_cloned_from_mutex_guard() {
        let status = DownloadStatus::new(sample_obj(), false, true, 123);
        let mutex = Mutex::new(status.clone());
        let guard = mutex.lock().await;

        let from_guard = DownloadStatus::from_mutex_guard(&guard);
        assert_eq!(from_guard, status);
    }

    #[test]
    fn download_status_can_be_deserialized_from_string() {
        let raw = r#"{"item":{"id":5,"url":"https://example.com/f","title":"f.zip","filesize":256},"paused":false,"downloading":true,"current_sz":64,"pausable":true}"#;

        let status: DownloadStatus = raw.to_string().into();
        assert_eq!(status.get_item().get_id(), 5);
        assert_eq!(status.get_item().get_file_name(), "f.zip");
        assert_eq!(status.get_current_sz(), 64);
        assert!(status.is_downloading());
        assert!(status.is_pausable());
    }

    #[test]
    fn download_info_display_format_is_stable() {
        let info = DownloadInfo::new(4, 1024);
        assert_eq!(info.to_string(), "id: 4, chunk_size: 1024");
    }
}
