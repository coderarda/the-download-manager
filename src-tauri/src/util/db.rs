use polodb_core::{bson::doc, CollectionT, Database};
use tokio::sync::Mutex;
use std::sync::Arc;

use super::download::DownloadStatus;

#[derive(Clone)]
pub struct Storage {
    db: Database,
}

impl Storage {
    pub fn new(path: &str) -> Self {
        let db = Database::open_path(path).unwrap();
        Self { db }
    }

    pub async fn save(&self, downloads: Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>>) {
        for d in downloads.lock().await.iter() {
            let d = d.lock().await;
            let coll = self.db.collection::<DownloadStatus>("downloads");
            coll.insert_one(d.clone()).unwrap();
        }        
    }

    pub async fn load(&self, downloads: Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>>) {
        let coll = self.db.collection::<DownloadStatus>("downloads");
        for doc in coll.find(doc! {}).run().unwrap() {
            let download: DownloadStatus = doc.unwrap();
            downloads.lock().await.push(Arc::new(Mutex::new(download)));
        }
    }
}
