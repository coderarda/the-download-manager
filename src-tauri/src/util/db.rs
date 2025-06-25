use sled::{Db, Error, IVec};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::download::DownloadStatus;

#[derive(Clone)]
pub struct Storage {
    tree: Arc<Mutex<Db>>,
}

impl Storage {
    pub fn new(path: &str) -> Self {
        let tree = sled::open(path).unwrap();
        Self { tree: Arc::new(Mutex::new(tree)) }
    }

    pub async fn save(&self, downloads: Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>>) {
        let db = self.tree.lock().await;
        for d in downloads.lock().await.iter() {
            let dw = DownloadStatus::from_mutex_guard(&d.lock().await);
            if dw.is_downloading() {
                let str = serde_json::to_string(&dw)
                    .unwrap();
                let bytes = str.as_bytes();
                let _ = db.insert("downloads", IVec::from(bytes));
            }
        }
    }

    pub async fn load(&self) -> Result<Vec<DownloadStatus>, Error> {
        let db = self.tree.lock().await;
        let res = db.open_tree("downloads")?;
        let mut result = Vec::new();
        for item in res.iter() {
            let (_key, value) = item?;
            // Deserialize from bytes to your DownloadObj
            let download: DownloadStatus = serde_json::from_slice(&value).unwrap();
            println!("{:?}", download);
            result.push(download);
        }
        Ok(result)
    }
}