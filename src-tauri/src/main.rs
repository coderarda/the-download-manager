// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use actix_web::{
    post,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use std::{
    cmp::Ordering,
    fs::File,
    io::Write,
    sync::Arc,
    time::Duration,
};
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

mod util;
use util::download::{DownloadInfo, DownloadObj, DownloadStatus};
use util::tauri_state::AppDownloadManager;
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

#[derive(Clone)]
struct WebServerState {
    handle: AppHandle,
}

#[tauri::command]
async fn pause_download(state: tauri::State<'_, AppDownloadManager>, id: u8) -> Result<(), String> {
    tokio::time::sleep(Duration::from_millis(100)).await;
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
    state: tauri::State<'_, AppDownloadManager>,
    handle: tauri::AppHandle,
    id: u8,
) -> Result<(), String> {
    for d in state.get_downloads().lock().await.iter() {
        if d.lock().await.get_item().get_id() == id {
            tokio::spawn(download_with_pause(d.clone(), handle.clone()));
            break;
        }
    }
    Ok(())
}

#[tauri::command]
async fn download(
    state: tauri::State<'_, AppDownloadManager>,
    handle: AppHandle,
    download: DownloadObj,
) -> Result<(), String> {
    for d in state.get_downloads().lock().await.iter_mut() {
        if d.lock().await.get_item().get_id() == download.get_id() {
            tokio::spawn(download_with_pause(d.clone(), handle.clone()));
            break;
        }
    }
    Ok(())
}

async fn download_with_pause(
    status: Arc<Mutex<DownloadStatus>>,
    handle: tauri::AppHandle,
) -> Result<(), ureq::Error> {
    println!("Download starting...");
    let h = handle.clone();
    let id = status.lock().await.get_item().get_id();
    status.lock().await.set_downloading();
    let starting_sz = status.lock().await.get_curr_size();
    let dir = tauri::api::path::download_dir()
        .unwrap()
        .join(status.lock().await.get_item().get_file_name());
    let mut file: Option<File>;
    match starting_sz.cmp(&0) {
        Ordering::Greater => {
            file = Some(
                std::fs::OpenOptions::new()
                    .append(true)
                    .open(dir.as_path())
                    .unwrap(),
            );
        }
        Ordering::Less => {
            panic!("Anomaly occurred! Canceling...");
            // Send event to frontend to delete file and try again
        }
        Ordering::Equal => {
            file = std::fs::File::create(dir.as_path()).ok();
        }
    }
    let req = ureq::get(&status.lock().await.get_item().get_url())
        .set("Range", &format!("bytes={starting_sz}-"))
        .call()?;

    if req.header("accept-ranges").unwrap().contains("bytes") {
        h.emit_all("downloadpauseinfo", true).unwrap();
    } else {
        h.emit_all("downloadpauseinfo", false).unwrap();
    }
    let mut curr_sz = starting_sz;
    let buf: &mut [u8] = &mut [0; 1024];
    let mut reader = req.into_reader();
    while curr_sz < status.lock().await.get_item().get_total_size() {
        if status.lock().await.is_paused() {
            break;
        }
        let sz = reader.read(buf).unwrap();
        let update = DownloadInfo::new(id, curr_sz);
        h.emit_all("ondownloadupdate", update).unwrap();
        match file {
            Some(ref mut f) => {
                f.write_all(buf).unwrap();
                curr_sz += sz as u64;
                status.lock().await.set_curr_size(curr_sz);
            }
            None => {
                println!("File not open!");
            }
        }
    }
    if curr_sz == status.lock().await.get_item().get_total_size() {
        let s = handle.state::<AppDownloadManager>();
        if s.get_downloads().lock().await.len() != 0 {
            s.get_downloads()
                .lock()
                .await
                .remove(status.lock().await.get_item().get_id() as usize);
            h.emit_all("ondownloadremove", id).unwrap();
        }
    }    
    Ok(())
}

async fn push_download_to_vec(download: &Arc<Mutex<DownloadStatus>>, handle: AppHandle) {
    let tauri_state: tauri::State<AppDownloadManager> = handle.state();
    tauri_state
        .get_downloads()
        .lock()
        .await
        .push(download.clone());
    handle
        .emit_all(
            "ondownload",
            serde_json::to_string(&download.lock().await.get_item()).unwrap(),
        )
        .expect("Message Could not be emitted.");
}

#[post("/")]
async fn listen_for_downloads(
    dw: String,
    data: web::Data<WebServerState>,
) -> std::io::Result<impl Responder> {
    let new_data = serde_json::from_str::<DownloadObj>(&dw).unwrap();
    // Get State
    let download = Arc::new(Mutex::new(DownloadStatus::new(new_data)));
    push_download_to_vec(&download, data.handle.clone()).await;
    Ok(HttpResponse::Ok())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let h = app.handle();
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
        .invoke_handler(tauri::generate_handler![pause_download, resume, download,])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
