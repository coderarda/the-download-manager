use actix_web::{post, web::Data, App, HttpResponse, HttpServer, Responder};
use regex::Regex;
use std::{io::Write, sync::Arc, time::Duration};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_http::reqwest;
use tauri_plugin_http::reqwest::header::CONTENT_LENGTH;
use tokio::sync::Mutex;

mod util;
use util::tauri_state::AppDownloadManager;
use util::{
    db::Storage,
    download::{DownloadInfo, DownloadObj, DownloadStatus},
};

#[derive(Clone)]
struct WebServerState {
    handle: AppHandle,
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
    let id = state.get_downloads().lock().await.len() as u8;
    let re = Regex::new(r"[^/]+$").unwrap();
    let title = re
        .find(&url)
        .map(|m| m.as_str().split('?').next().unwrap().to_string())
        .unwrap();
    Ok(DownloadObj::new(id, url, title, total_size))
}

#[tauri::command]
async fn pause_download(state: tauri::State<'_, AppDownloadManager>, id: u8) -> Result<(), String> {
    for d in state.get_downloads().lock().await.iter_mut() {
        if d.lock().await.get_item().get_id() == id {
            d.lock().await.set_pause();
            break;
        }
    }
    Ok(())
}

#[tauri::command]
async fn resume(state: tauri::State<'_, AppDownloadManager>, id: u8) -> Result<(), String> {
    for d in state.get_downloads().lock().await.iter() {
        if d.lock().await.get_item().get_id() == id {
            d.lock().await.set_downloading();
            break;
        }
    }
    Ok(())
}

#[tauri::command]
async fn remove_download(handle: AppHandle, id: u8) -> Result<(), String> {
    handle
        .state::<AppDownloadManager>()
        .get_downloads()
        .lock()
        .await
        .remove(id as usize);
    handle.emit("download_removed", id).unwrap();
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
    for d in state.get_downloads().lock().await.iter_mut() {
        if d.lock().await.get_item().get_id() == download.get_item().get_id() {
            tokio::spawn(download_item(d.clone(), handle.clone()));
            break;
        }
    }
    Ok(())
}

#[tauri::command]
async fn load_downloads(app: AppHandle) -> Result<Vec<DownloadStatus>, String> {
    let db_path = app
        .path()
        .app_local_data_dir()
        .unwrap()
        .join("downloads.db");
    let st = Storage::new(db_path.to_str().unwrap());
    let downloads = st.load().await.unwrap();
    Ok(downloads)
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

    // Ensure unique filename by appending or incrementing (1), (2), etc., if necessary
    let mut counter = 1;
    while std::fs::metadata(&filepath).is_ok() {
        let file_stem = dw_item
            .get_file_name()
            .rsplit_once('.')
            .map_or(dw_item.get_file_name().as_str(), |(stem, _)| stem);
        let extension = dw_item.get_file_name().rsplit_once('.').map(|(_, ext)| ext);

        // Check if the filename already ends with a number in parentheses
        let new_stem =
            if let Some(captures) = Regex::new(r"^(.*)\((\d+)\)$").unwrap().captures(file_stem) {
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

    let mut file = file_opts.create(true).write(true).open(&filepath).unwrap();

    let client = reqwest::Client::new();
    // Set Range request header to resume download
    let mut req = client
        .get(status.lock().await.get_item().get_url())
        .send()
        .await?;

    let mut curr_sz = 0;
    let mut count = 0;

    while let Some(buf) = req.chunk().await? {
        while status.lock().await.is_paused() {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        file.write_all(&buf).unwrap();
        curr_sz += buf.len() as u64;

        if count % 50 == 0 {
            let update = DownloadInfo::new(id, curr_sz);
            handle.emit("ondownloadupdate", update).unwrap();
        }
        count += 1;
    }
    Ok(())
}

async fn push_download_to_vec(download: &Arc<Mutex<DownloadStatus>>, handle: AppHandle) {
    let tauri_state: tauri::State<AppDownloadManager> = handle.state();
    download.lock().await.get_item()
        .set_id(tauri_state.get_downloads().lock().await.len() as u8);
    
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
    let download = Arc::new(Mutex::new(DownloadStatus::new(new_data, false, false)));
    push_download_to_vec(&download, data.handle.clone()).await;
    Ok(HttpResponse::Ok())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let h = app.handle().clone();
            let handle_clone = h.clone();
            let handle = handle_clone.clone();
            let window = app.get_webview_window("main").unwrap();
            // Load downloads from db
            let db_path = app
                .path()
                .app_local_data_dir()
                .unwrap()
                .join("downloads.db");

            window.on_window_event(move |event| {
                let new_handle = handle_clone.clone();
                let new_state = new_handle.state::<AppDownloadManager>();
                if let tauri::WindowEvent::CloseRequested { .. } = event {
                    // Save downloads to db
                    let s = Storage::new(db_path.to_str().unwrap());
                    tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(async move {
                            s.save(new_state.get_downloads()).await;
                        });
                }
            });
            tauri::async_runtime::spawn(
                HttpServer::new(move || {
                    App::new()
                        .service(listen_for_downloads)
                        .app_data(Data::new(WebServerState {
                            handle: handle.clone(),
                        }))
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
            load_downloads
        ])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
