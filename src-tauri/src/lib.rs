use actix_web::{post, web::Data, App, HttpResponse, HttpServer, Responder};
use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::{io::Write, path::{Path, PathBuf}, sync::Arc};
use tauri::http::header::{ACCEPT_RANGES, RANGE};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_http::reqwest;
use tauri_plugin_http::reqwest::header::CONTENT_LENGTH;
use tokio::sync::Mutex;

mod util;
use util::download::{DownloadInfo, DownloadObj, DownloadStatus};
use util::tauri_state::AppDownloadManager;

#[derive(Clone)]
struct WebServerState {
    handle: AppHandle,
}

#[tauri::command]
async fn load_downloads(handle: AppHandle) -> Result<Vec<DownloadStatus>, String> {
    let p = handle.path().app_data_dir().unwrap().join("downloads.json");
    println!("{:?}", p);
    let mut data = String::new();
    if p.exists() {
        match std::fs::OpenOptions::new().read(true).open(&p) {
            Ok(mut file) => {
                if let Err(e) = file.read_to_string(&mut data) {
                    return Err(format!("Failed to read downloads.json: {}", e));
                }
            }
            Err(e) => {
                return Err(format!("Failed to open downloads.json: {}", e));
            }
        }
    } else {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
    }
    if data.is_empty() {
        data = "[]".to_string();
    }
    let downloads: Vec<DownloadStatus> = match serde_json::from_str(&data) {
        Ok(d) => d,
        Err(e) => return Err(format!("Failed to parse downloads.json: {}", e)),
    };

    let state: tauri::State<AppDownloadManager> = handle.state();
    let dw_arc = state.get_downloads();
    let mut dw_list = dw_arc.lock().await;
    dw_list.clear();
    let mut max_id: u32 = 0;
    let download_dir = handle.path().download_dir().unwrap();
    for dw in &downloads {
        let mut dw_clone = dw.clone();
        let filepath = download_dir.join(dw_clone.get_item().get_file_name());
        if let Ok(metadata) = std::fs::metadata(&filepath) {
            let on_disk = metadata.len();
            if on_disk > 0 {
                dw_clone.set_current_sz(on_disk);
            }
        }
        let wrapped = Arc::new(Mutex::new(dw_clone.clone()));
        let id = dw.get_item().get_id();
        if id >= max_id {
            max_id = id + 1;
        }
        dw_list.push(wrapped);
        handle.emit("downloadfirstupdate", &dw_clone).unwrap();
    }
    state.set_next_id(max_id);

    Ok(downloads)
}

fn persist_downloads(path: &Path, downloads: &[DownloadStatus]) -> Result<(), String> {
    let json = serde_json::to_string(downloads).map_err(|e| format!("Failed to serialize: {}", e))?;
    std::fs::write(path, json).map_err(|e| format!("Failed to write: {}", e))?;
    Ok(())
}

async fn find_download_by_id(
    downloads: &[Arc<Mutex<DownloadStatus>>],
    target_id: u32,
) -> Option<Arc<Mutex<DownloadStatus>>> {
    for download in downloads {
        if download.lock().await.get_item().get_id() == target_id {
            return Some(download.clone());
        }
    }
    None
}

async fn find_download_index_by_id(
    downloads: &[Arc<Mutex<DownloadStatus>>],
    target_id: u32,
) -> Option<usize> {
    for (index, download) in downloads.iter().enumerate() {
        if download.lock().await.get_item().get_id() == target_id {
            return Some(index);
        }
    }
    None
}

#[tauri::command]
async fn get_download_info(
    state: tauri::State<'_, AppDownloadManager>,
    mut url: String,
) -> Result<DownloadObj, String> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .unwrap();
    let resp = client.head(&url).send().await.unwrap();
    url = resp.url().to_string();
    let total_size = resp
        .headers()
        .get(CONTENT_LENGTH)
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let id = state.get_next_id();
    let re = Regex::new(r"[^/]+$").unwrap();
    let title = re
        .find(&url)
        .map(|m| m.as_str().split('?').next().unwrap().to_string())
        .unwrap();
    Ok(DownloadObj::new(id, url, title, total_size))
}

#[tauri::command]
async fn pause_download(state: tauri::State<'_, AppDownloadManager>, id: u32) -> Result<(), String> {
    for d in state.get_downloads().lock().await.iter_mut() {
        if d.lock().await.get_item().get_id() == id {
            d.lock().await.set_pause();
            break;
        }
    }
    Ok(())
}

#[tauri::command]
async fn resume(
    app_handle: AppHandle,
    state: tauri::State<'_, AppDownloadManager>,
    download: DownloadStatus,
) -> Result<(), String> {
    let dw_arc = state.get_downloads();
    let mut dw_list = dw_arc.lock().await;
    let target_id = download.get_item().get_id();
    let found = find_download_by_id(&dw_list, target_id).await;
    if let Some(d) = found {
        d.lock().await.set_downloading();
        tokio::spawn(download_item(d, app_handle));
    } else {
        let wrapped = Arc::new(Mutex::new(download));
        tokio::spawn(download_item(wrapped.clone(), app_handle));
        dw_list.push(wrapped);
    }
    Ok(())
}

#[tauri::command]
async fn remove_download(handle: AppHandle, id: u32) -> Result<(), String> {
    let dw_arc = handle.state::<AppDownloadManager>().get_downloads();
    let file_name: Option<String>;
    {
        let mut dw_list = dw_arc.lock().await;
        let idx = find_download_index_by_id(&dw_list, id).await;
        if let Some(i) = idx {
            file_name = Some(dw_list[i].lock().await.get_item().get_file_name().clone());
            dw_list.remove(i);
            let mut dws = Vec::new();
            for d in dw_list.iter() {
                let dw = DownloadStatus::from_mutex_guard(&d.lock().await);
                dws.push(dw);
            }
            let path = handle
                .path()
                .app_data_dir()
                .unwrap()
                .join("downloads.json");
            persist_downloads(&path, &dws)?;
        } else {
            file_name = None;
        }
    }
    handle.emit("download_removed", id).unwrap();
    if let Some(name) = file_name {
        let filepath = handle.path().download_dir().unwrap().join(name);
        if filepath.exists() {
            let _ = std::fs::remove_file(&filepath);
        }
    }
    Ok(())
}

#[tauri::command]
async fn download_manually_from_url(
    state: tauri::State<'_, AppDownloadManager>,
    handle: AppHandle,
    download: DownloadObj,
) -> Result<(), String> {
    let download_status = Arc::new(Mutex::new(DownloadStatus::new(
        download.clone(),
        false,
        false,
        0,
    )));
    state
        .get_downloads()
        .lock()
        .await
        .push(download_status.clone());
    for d in state.get_downloads().lock().await.iter_mut() {
        if d.lock().await.get_item().get_id() == download.get_id() {
            tokio::spawn(download_item(d.clone(), handle.clone()));
            break;
        }
    }
    Ok(())
}

#[tauri::command]
async fn download(
    state: tauri::State<'_, AppDownloadManager>,
    handle: AppHandle,
    download: DownloadStatus,
) -> Result<(), String> {
    let dw_arc = state.get_downloads();
    let mut dw_list = dw_arc.lock().await;
    let target_id = download.get_item().get_id();
    let found = find_download_by_id(&dw_list, target_id).await;
    if let Some(d) = found {
        tokio::spawn(download_item(d, handle));
    } else {
        let wrapped = Arc::new(Mutex::new(download));
        tokio::spawn(download_item(wrapped.clone(), handle));
        dw_list.push(wrapped);
    }
    Ok(())
}

trait DownloadEventEmitter {
    fn emit_download_pause_info(&self, id: u32, pausable: bool);
    fn emit_download_update(&self, update: DownloadInfo);
}

impl DownloadEventEmitter for AppHandle {
    fn emit_download_pause_info(&self, id: u32, pausable: bool) {
        self.emit("downloadpauseinfo", (id, pausable)).unwrap();
    }

    fn emit_download_update(&self, update: DownloadInfo) {
        self.emit("ondownloadupdate", update).unwrap();
    }
}

fn split_file_name(file_name: &str) -> (&str, Option<&str>) {
    file_name
        .rsplit_once('.')
        .map_or((file_name, None), |(stem, extension)| {
            (stem, Some(extension))
        })
}

fn split_counter_suffix(file_stem: &str) -> Option<(&str, u32)> {
    if !file_stem.ends_with(')') {
        return None;
    }

    let start_idx = file_stem.rfind('(')?;
    let number = file_stem[start_idx + 1..file_stem.len() - 1].parse().ok()?;
    Some((&file_stem[..start_idx], number))
}

fn build_indexed_file_name(original_file_name: &str, counter: u32) -> String {
    let (file_stem, extension) = split_file_name(original_file_name);
    let indexed_stem = if let Some((base_stem, current_number)) = split_counter_suffix(file_stem) {
        format!("{}({})", base_stem, current_number + counter)
    } else {
        format!("{}({})", file_stem, counter)
    };

    match extension {
        Some(ext) => format!("{}.{}", indexed_stem, ext),
        None => indexed_stem,
    }
}

fn resolve_available_filepath(download_dir: &Path, original_file_name: &str) -> PathBuf {
    let mut counter = 1;
    loop {
        let candidate = download_dir.join(build_indexed_file_name(original_file_name, counter));
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}

fn resolve_download_filepath(download_dir: &Path, item: &DownloadObj) -> (PathBuf, Option<String>) {
    let filepath = download_dir.join(item.get_file_name());

    if !filepath.exists() {
        return (filepath, None);
    }

    let existing_len_matches_total = std::fs::metadata(&filepath)
        .map(|metadata| metadata.len() == item.get_total_size())
        .unwrap_or(false);

    if !existing_len_matches_total {
        return (filepath, None);
    }

    let new_path = resolve_available_filepath(download_dir, item.get_file_name());
    let new_name = new_path.file_name().map(|name| name.to_string_lossy().to_string());
    (new_path, new_name)
}

fn open_download_file(filepath: &Path) -> File {
    let mut file_opts = std::fs::OpenOptions::new();
    if filepath.exists() {
        file_opts.append(true).open(filepath).unwrap()
    } else {
        file_opts.create(true).write(true).open(filepath).unwrap()
    }
}

fn read_file_size(file: &File) -> u64 {
    file.metadata().map(|metadata| metadata.len()).unwrap_or(0)
}

async fn download_item(
    status: Arc<Mutex<DownloadStatus>>,
    handle: AppHandle,
) -> Result<(), reqwest::Error> {
    let download_dir = handle.path().download_dir().unwrap();
    download_item_with_emitter(status, download_dir, &handle).await
}

async fn download_item_with_emitter<E: DownloadEventEmitter>(
    status: Arc<Mutex<DownloadStatus>>,
    download_dir: PathBuf,
    emitter: &E,
) -> Result<(), reqwest::Error> {
    println!("Download starting...");
    let download_item = {
        let mut status_guard = status.lock().await;
        status_guard.set_downloading();
        status_guard.get_item()
    };
    let id = download_item.get_id();
    let (filepath, updated_name) = resolve_download_filepath(&download_dir, &download_item);

    if let Some(new_name) = updated_name {
        status.lock().await.set_item_file_name(new_name);
    }

    let mut f = open_download_file(&filepath);

    let client = reqwest::Client::new();
    let head = client.head(download_item.get_url()).send().await?;

    // Set Range request header to resume download
    let is_pausable = head.headers().contains_key(ACCEPT_RANGES);
    status.lock().await.set_pausable(is_pausable);
    emitter.emit_download_pause_info(id, is_pausable);

    let len = read_file_size(&f);
    let mut req = client
        .get(download_item.get_url())
        .header(RANGE, format!("bytes={}-", len))
        .send()
        .await?;

    let mut curr_sz = status.lock().await.get_current_sz();
    let mut count = 0;
    let mut was_paused = false;

    while let Some(buf) = req.chunk().await? {
        if status.lock().await.is_paused() {
            status.lock().await.set_current_sz(curr_sz);
            was_paused = true;
            break;
        }

        f.write_all(&buf).unwrap();
        curr_sz += buf.len() as u64;
        status.lock().await.set_current_sz(curr_sz);

        if count % 50 == 0 {
            let update = DownloadInfo::new(id, curr_sz);
            emitter.emit_download_update(update);
        }
        count += 1;
    }

    if !was_paused {
        let total = download_item.get_total_size();
        status.lock().await.set_current_sz(total);
    }
    Ok(())
}

async fn push_download_to_vec(download: &Arc<Mutex<DownloadStatus>>, handle: AppHandle) {
    let tauri_state: tauri::State<AppDownloadManager> = handle.state();
    let id = tauri_state.get_next_id();
    download.lock().await.set_item_id(id);

    tauri_state
        .get_downloads()
        .lock()
        .await
        .push(download.clone());
    handle
        .emit(
            "ondownload",
            DownloadStatus::from_mutex_guard(&download.lock().await),
        )
        .expect("Message Could not be emitted.");
}

#[post("/")]
async fn listen_for_downloads(
    dw: String,
    data: Data<WebServerState>,
) -> std::io::Result<impl Responder> {
    let new_data = serde_json::from_str::<DownloadObj>(&dw)?;
    let download = Arc::new(Mutex::new(DownloadStatus::new(new_data, false, false, 0)));
    push_download_to_vec(&download, data.handle.clone()).await;
    Ok(HttpResponse::Ok())
}

async fn get_from_arc_vec(vec: Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>>) -> Vec<DownloadStatus> {
    let mut dws = Vec::new();
    for d in vec.lock().await.iter() {
        let dw = DownloadStatus::from_mutex_guard(&d.lock().await);
        dws.push(dw);
    }
    dws
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let h = app.handle().clone();
            // Listen for window close event
            let main_window = app.handle().get_window("main");
            if let Some(window) = main_window {
                let another_handle = h.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        println!("App is closing. Perform cleanup here.");
                        let handle = another_handle.clone();
                        let downloads_arc =
                            handle.state::<AppDownloadManager>().get_downloads();
                        let downloads = tauri::async_runtime::block_on(async {
                            get_from_arc_vec(downloads_arc).await
                        });
                        let path = handle
                            .path()
                            .app_data_dir()
                            .unwrap()
                            .join("downloads.json");
                        let _ = persist_downloads(&path, &downloads);
                    }
                });
            }
            // Load downloads from db
            tauri::async_runtime::spawn(
                HttpServer::new(move || {
                    App::new()
                        .service(listen_for_downloads)
                        .app_data(Data::new(WebServerState { handle: h.clone() }))
                })
                .bind(("localhost", 4000))?
                .run(),
            );
            Ok(())
        })
        .manage(AppDownloadManager::new(Arc::new(Mutex::new(Vec::<
            Arc<Mutex<DownloadStatus>>,
        >::new(
        )))))
        .invoke_handler(tauri::generate_handler![
            pause_download,
            resume,
            download,
            get_download_info,
            download_manually_from_url,
            remove_download,
            load_downloads,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::{
        download_item_with_emitter, DownloadEventEmitter, DownloadInfo, DownloadObj, DownloadStatus,
    };
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex as StdMutex};
    use std::thread::JoinHandle;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::sync::Mutex;

    #[derive(Default)]
    struct TestEventEmitter {
        pause_events: StdMutex<Vec<(u32, bool)>>,
        update_events: StdMutex<Vec<String>>,
    }

    impl DownloadEventEmitter for TestEventEmitter {
        fn emit_download_pause_info(&self, id: u32, pausable: bool) {
            self.pause_events.lock().unwrap().push((id, pausable));
        }

        fn emit_download_update(&self, update: DownloadInfo) {
            self.update_events.lock().unwrap().push(update.to_string());
        }
    }

    fn create_temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{}_{}", prefix, nanos));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn spawn_test_server(
        head_content_length: usize,
        get_body: Vec<u8>,
        includes_accept_ranges: bool,
    ) -> (String, Arc<StdMutex<Vec<String>>>, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let requests = Arc::new(StdMutex::new(Vec::<String>::new()));
        let requests_for_thread = requests.clone();

        let handle = std::thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buf = [0_u8; 2048];
                let bytes_read = stream.read(&mut buf).unwrap();
                let request = String::from_utf8_lossy(&buf[..bytes_read]);
                requests_for_thread.lock().unwrap().push(request.to_string());

                let accept_ranges_header = if includes_accept_ranges {
                    "Accept-Ranges: bytes\r\n"
                } else {
                    ""
                };

                if request.starts_with("HEAD") {
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n{}Connection: close\r\n\r\n",
                        head_content_length,
                        accept_ranges_header
                    );
                    stream.write_all(response.as_bytes()).unwrap();
                } else if request.starts_with("GET") {
                    let headers = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n{}Connection: close\r\n\r\n",
                        get_body.len(),
                        accept_ranges_header
                    );
                    stream.write_all(headers.as_bytes()).unwrap();
                    stream.write_all(&get_body).unwrap();
                } else {
                    panic!("unexpected request: {}", request);
                }
            }
        });

        (format!("http://{}/test.bin", addr), requests, handle)
    }

    #[tokio::test]
    async fn download_item_writes_file_and_updates_status() {
        let payload = b"hello from download item test".to_vec();
        let (url, _requests, server_handle) =
            spawn_test_server(payload.len(), payload.clone(), true);
        let temp_dir = create_temp_dir("download_item_test");
        let file_name = "test.bin".to_string();
        let status = Arc::new(Mutex::new(DownloadStatus::new(
            DownloadObj::new(42, url, file_name.clone(), payload.len() as u64),
            false,
            false,
            0,
        )));
        let emitter = TestEventEmitter::default();

        let result = download_item_with_emitter(status.clone(), temp_dir.clone(), &emitter).await;

        server_handle.join().unwrap();
        assert!(result.is_ok());

        let written_file = temp_dir.join(file_name);
        assert_eq!(std::fs::read(&written_file).unwrap(), payload);

        let status_guard = status.lock().await;
        assert!(status_guard.is_downloading());
        assert_eq!(status_guard.get_current_sz(), payload.len() as u64);
        assert!(status_guard.is_pausable());
        drop(status_guard);

        let pause_events = emitter.pause_events.lock().unwrap();
        assert_eq!(pause_events.as_slice(), &[(42, true)]);
        drop(pause_events);

        let update_events = emitter.update_events.lock().unwrap();
        assert_eq!(update_events.len(), 1);
        drop(update_events);

        std::fs::remove_file(written_file).unwrap();
        std::fs::remove_dir_all(temp_dir).unwrap();
    }

    #[tokio::test]
    async fn download_item_sets_not_pausable_without_accept_ranges() {
        let payload = b"download without range support".to_vec();
        let (url, _requests, server_handle) =
            spawn_test_server(payload.len(), payload.clone(), false);
        let temp_dir = create_temp_dir("download_item_non_pausable_test");
        let file_name = "non_pausable_test.bin".to_string();
        let status = Arc::new(Mutex::new(DownloadStatus::new(
            DownloadObj::new(7, url, file_name.clone(), payload.len() as u64),
            false,
            false,
            0,
        )));
        let emitter = TestEventEmitter::default();

        let result = download_item_with_emitter(status.clone(), temp_dir.clone(), &emitter).await;

        server_handle.join().unwrap();
        assert!(result.is_ok());

        let written_file = temp_dir.join(file_name);
        assert_eq!(std::fs::read(&written_file).unwrap(), payload);

        let status_guard = status.lock().await;
        assert!(status_guard.is_downloading());
        assert_eq!(status_guard.get_current_sz(), payload.len() as u64);
        assert!(!status_guard.is_pausable());
        drop(status_guard);

        let pause_events = emitter.pause_events.lock().unwrap();
        assert_eq!(pause_events.as_slice(), &[(7, false)]);
        drop(pause_events);

        std::fs::remove_file(written_file).unwrap();
        std::fs::remove_dir_all(temp_dir).unwrap();
    }

    #[tokio::test]
    async fn download_item_uses_file_size_for_range_and_appends_existing_file() {
        let initial_bytes = b"hello";
        let appended_bytes = b" world";
        let expected_content = [initial_bytes.as_slice(), appended_bytes.as_slice()].concat();

        let (url, requests, server_handle) =
            spawn_test_server(expected_content.len(), appended_bytes.to_vec(), true);
        let temp_dir = create_temp_dir("download_item_resume_test");
        let file_name = "resume_test.bin".to_string();
        let existing_file = temp_dir.join(&file_name);
        std::fs::write(&existing_file, initial_bytes).unwrap();

        let status = Arc::new(Mutex::new(DownloadStatus::new(
            DownloadObj::new(99, url, file_name.clone(), expected_content.len() as u64),
            false,
            false,
            initial_bytes.len() as u64,
        )));
        let emitter = TestEventEmitter::default();

        let result = download_item_with_emitter(status.clone(), temp_dir.clone(), &emitter).await;

        server_handle.join().unwrap();
        assert!(result.is_ok());

        assert_eq!(std::fs::read(&existing_file).unwrap(), expected_content);

        let reqs = requests.lock().unwrap();
        let get_request = reqs
            .iter()
            .find(|request| request.starts_with("GET"))
            .expect("Expected GET request to be captured")
            .to_lowercase();
        assert!(
            get_request.contains("range: bytes=5-"),
            "Expected resume range header in request, got: {}",
            get_request
        );
        drop(reqs);

        let status_guard = status.lock().await;
        assert_eq!(status_guard.get_current_sz(), expected_content.len() as u64);
        assert!(status_guard.is_pausable());
        drop(status_guard);

        std::fs::remove_file(existing_file).unwrap();
        std::fs::remove_dir_all(temp_dir).unwrap();
    }
}
