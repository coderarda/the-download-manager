use std::sync::Arc;
use pickledb::PickleDb;
use tokio::sync::Mutex;
use super::download::DownloadStatus;

pub struct Storage {
    db: PickleDb,
    downloads: Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>>,
}

impl Storage {
    pub fn new(path: &str, downloads: Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>>) -> Self {
        let db = PickleDb::new(
            path,
            pickledb::PickleDbDumpPolicy::AutoDump,
            pickledb::SerializationMethod::Json,
        );
        Self { db, downloads }
    }

    pub async fn save(&mut self) {
        self.db.lcreate("downloads").unwrap();
        for d in self.downloads.lock().await.iter() {
            let download = d.lock().await.get_item();
            let st = DownloadStatus::new(download, false, false);
            self.db
                .ladd("downloads", &st)
                .expect("Failed to add download to database");
        }
    }
}