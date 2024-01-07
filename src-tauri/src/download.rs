    use serde::{Deserialize, Serialize};

    #[derive(Clone, Serialize, Deserialize)]
    pub struct DownloadObj {
        id: u32,
        url: String,
        title: String,
        filesize: u64,
    }

    impl DownloadObj {
        pub fn get_url(&self) -> &String {
            &self.url
        }

        pub fn get_file_name(&self) -> &String {
            &self.title
        }

        pub fn get_id(&self) -> u32 {
            self.id
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


