use actix_web::{post, web::Data, App, HttpResponse, HttpServer, Responder};
use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::{io::Write, sync::Arc};
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

async fn save_downloads(
    handle: AppHandle,
    downloads: Arc<Mutex<Vec<Arc<Mutex<DownloadStatus>>>>>,
) -> Result<(), String> {
    let path = handle.path().app_data_dir().unwrap().join("downloads.json");
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    let new_downloads = get_from_arc_vec(downloads).await;
    let json = serde_json::to_string(&new_downloads).map_err(|e| format!("Failed to serialize: {}", e))?;
    f.write_all(json.as_bytes())
        .map_err(|e| format!("Failed to write: {}", e))?;
    Ok(())
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
    let mut found = None;
    for d in dw_list.iter() {
        if d.lock().await.get_item().get_id() == target_id {
            found = Some(d.clone());
            break;
        }
    }
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
        let mut idx = None;
        for (i, d) in dw_list.iter().enumerate() {
            if d.lock().await.get_item().get_id() == id {
                idx = Some(i);
                break;
            }
        }
        if let Some(i) = idx {
            file_name = Some(dw_list[i].lock().await.get_item().get_file_name().clone());
            dw_list.remove(i);
            let mut dws = Vec::new();
            for d in dw_list.iter() {
                let dw = DownloadStatus::from_mutex_guard(&d.lock().await);
                dws.push(dw);
            }
            let json =
                serde_json::to_string(&dws).map_err(|e| format!("Failed to serialize: {}", e))?;
            let path = handle
                .path()
                .app_data_dir()
                .unwrap()
                .join("downloads.json");
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&path)
                .map_err(|e| format!("Failed to open file: {}", e))?;
            f.write_all(json.as_bytes())
                .map_err(|e| format!("Failed to write: {}", e))?;
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
    let mut found = None;
    for d in dw_list.iter() {
        if d.lock().await.get_item().get_id() == target_id {
            found = Some(d.clone());
            break;
        }
    }
    if let Some(d) = found {
        tokio::spawn(download_item(d, handle));
    } else {
        let wrapped = Arc::new(Mutex::new(download));
        tokio::spawn(download_item(wrapped.clone(), handle));
        dw_list.push(wrapped);
    }
    Ok(())
}

async fn download_item(
    status: Arc<Mutex<DownloadStatus>>,
    handle: AppHandle,
) -> Result<(), reqwest::Error> {
    println!("Download starting...");
    let mut filepath = handle
        .path()
        .download_dir()
        .unwrap()
        .join(status.lock().await.get_item().get_file_name());
    let id = status.lock().await.get_item().get_id();
    status.lock().await.set_downloading();
    let mut file_opts = std::fs::OpenOptions::new();
    let dw_item = status.lock().await.get_item();

    let client = reqwest::Client::new();
    let head = client
        .head(status.lock().await.get_item().get_url())
        .send()
        .await?;

    let mut counter = 1;
    if filepath.exists()
        && std::fs::metadata(&filepath).unwrap().len()
            == status.lock().await.get_item().get_total_size()
    {
        while std::fs::metadata(&filepath).is_ok() {
            let file_stem = dw_item
                .get_file_name()
                .rsplit_once('.')
                .map_or(dw_item.get_file_name().as_str(), |(stem, _)| stem);
            let extension = dw_item.get_file_name().rsplit_once('.').map(|(_, ext)| ext);

            // Check if the filename already ends with a number in parentheses
            let new_stem = if let Some(captures) =
                Regex::new(r"^(.*)\((\d+)\)$").unwrap().captures(file_stem)
            {
                let base_stem = captures.get(1).unwrap().as_str();
                let current_number: u32 = captures.get(2).unwrap().as_str().parse().unwrap();
                format!("{}({})", base_stem, current_number + 1)
            } else {
                format!("{}({})", file_stem, counter)
            };

            let new_name = match extension {
                Some(ext) => format!("{}.{}", new_stem, ext),
                None => new_stem,
            };
            filepath = handle.path().download_dir().unwrap().join(new_name);
            counter += 1;
        }
        let new_name = filepath.file_name().unwrap().to_string_lossy().to_string();
        status.lock().await.set_item_file_name(new_name);
    }
    let file: Option<File>;
    if filepath.exists() {
        file = Some(file_opts.append(true).open(&filepath).unwrap());
    } else {
        file = Some(file_opts.create(true).write(true).open(&filepath).unwrap());
    }

    let mut f = match file {
        Some(f) => f,
        None => {
            panic!("File could not be created/opened");
        }
    };

    // Set Range request header to resume download
    if head.headers().contains_key(ACCEPT_RANGES) {
        status.lock().await.set_pausable(true);
        handle.emit("downloadpauseinfo", (id, true)).unwrap();
    } else {
        status.lock().await.set_pausable(false);
        handle.emit("downloadpauseinfo", (id, false)).unwrap();
    }
    let mut len = 0;
    if let Ok(metadata) = f.metadata() {
        if metadata.len() > 0 {
            len = metadata.len();
        }
    }
    let mut req = client
        .get(status.lock().await.get_item().get_url())
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
            handle.emit("ondownloadupdate", update).unwrap();
        }
        count += 1;
    }

    if !was_paused {
        let total = status.lock().await.get_item().get_total_size();
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
                        if let Ok(json) = serde_json::to_string(&downloads) {
                            let _ = std::fs::write(&path, json);
                        }
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
