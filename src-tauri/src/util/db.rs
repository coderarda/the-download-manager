use super::download::DownloadStatus;
use pickledb::PickleDb;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Storage {
    db: Arc<Mutex<PickleDb>>,
}

impl Storage {
    pub fn new(path: &str) -> Self {
        let db = Arc::new(Mutex::new(PickleDb::new(
            path,
            pickledb::PickleDbDumpPolicy::AutoDump,
            pickledb::SerializationMethod::Json,
        )));
        Self { db }
    }

    pub async fn save(&mut self, downloads: Arc<tokio::sync::Mutex<Vec<Arc<tokio::sync::Mutex<DownloadStatus>>>>>) {
        if !self.db.lock().unwrap().exists("downloads") {
            self.db.lock().unwrap().lcreate("downloads").unwrap();
        }
        for d in downloads.lock().await.iter() {
            let download = d.lock().await.get_item();
            let st = DownloadStatus::new(download, false, false);
            self.db
                .lock()
                .unwrap()
                .ladd("downloads", &st)
                .expect("Failed to add download to database");
        }
    }

    pub async fn load(&mut self, downloads: Arc<tokio::sync::Mutex<Vec<Arc<tokio::sync::Mutex<DownloadStatus>>>>>) {
        for i in 0..=self.db.lock().unwrap().llen("downloads") {
            let dw = self
                .db
                .lock()
                .unwrap()
                .lget::<DownloadStatus>("downloads", i as usize)
                .unwrap();
            let mutex = Arc::new(tokio::sync::Mutex::new(dw));
            downloads.lock().await.push(mutex.clone());
        }
    }
}
