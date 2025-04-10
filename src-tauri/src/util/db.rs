use std::sync::Arc;
use pickledb::PickleDb;
use tokio::sync::Mutex;
use super::download::DownloadStatus;


#[derive(Clone)]
pub struct Storage {
    db: Arc<Mutex<PickleDb>>,
    downloads: Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>>,
}

impl Storage {
    pub fn new(path: &str, downloads: Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>>) -> Self {
        let db = Arc::new(Mutex::new(PickleDb::new(
            path,
            pickledb::PickleDbDumpPolicy::AutoDump,
            pickledb::SerializationMethod::Json,
        )));
        Self { db, downloads }
    }

    pub async fn save(&mut self) {
        self.db.lock().await.lcreate("downloads").unwrap();
        for d in self.downloads.lock().await.iter() {
            let download = d.lock().await.get_item();
            let st = DownloadStatus::new(download, false, false);
            self.db.lock().await
                .ladd("downloads", &st)
                .expect("Failed to add download to database");
        }
    }

    pub async fn load(&mut self) {
        for i in 0..self.db.lock().await.llen("downloads") {
            let dw = self.db.lock().await.lget::<DownloadStatus>("downloads", i as usize);
            let mutex = Arc::new(Mutex::new(dw.unwrap()));
            self.downloads.lock().await.push(mutex.clone());
        } 
    }
}