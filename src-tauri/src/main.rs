// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use actix_web::{
    post,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use futures_util::StreamExt;
use reqwest::header::RANGE;
use std::{fs::File, io::Write, sync::Arc, time::Duration};
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;
mod download;
use download::{DownloadInfo, DownloadObj, DownloadStatus};

mod tauri_state;
use tauri_state::TauriState;
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

#[derive(Clone)]
struct AppState {
    handle: AppHandle,
}

#[tauri::command]
async fn pause_download(state: tauri::State<'_, TauriState>, id: u32) -> Result<(), String> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    let vec_res = state.downloads.try_lock();
    match vec_res {
        Ok(mut vec) => {
            for d in vec.iter_mut() {
                if d.lock().await.get_item().get_id() == id {
                    d.lock().await.set_pause();
                    break;
                }
            }
        }
        Err(err) => {
            println!("Error occured: {err}");
            ()
        }
    }
    Ok(())
}

#[tauri::command]
async fn resume(
    state: tauri::State<'_, TauriState>,
    handle: tauri::AppHandle,
    id: u32,
) -> Result<(), String> {
    for d in state.downloads.lock().await.iter_mut() {
        if d.lock().await.get_item().get_id() == id {
            tokio::spawn(download_self(d.clone(), handle.clone()));
            break;
        }
    }
    Ok(())
}

#[tauri::command]
async fn download(
    state: tauri::State<'_, TauriState>,
    handle: AppHandle,
    download: DownloadObj,
) -> Result<(), String> {
    for d in state.downloads.lock().await.iter_mut() {
        if d.lock().await.get_item().get_id() == download.get_id() {
            d.lock().await.set_item(download);
            tokio::spawn(download_self(d.clone(), handle.clone()));
            break;
        }
    }
    Ok(())
}

async fn download_self(
    status_obj: Arc<Mutex<DownloadStatus>>,
    handle: tauri::AppHandle,
) -> Result<(), reqwest::Error> {
    println!("Download starting...");
    let h = handle.clone();
    let id = status_obj.lock().await.get_item().get_id();
    status_obj.lock().await.set_downloading();
    let arg1 = status_obj.lock().await.get_size();
    let size = status_obj.lock().await.get_item().get_total_size();
    let client = reqwest::Client::new();
    let res = client
        .get(status_obj.lock().await.get_item().get_url())
        .header(RANGE, format!("bytes={arg1}-{size}"))
        .send()
        .await?;
    tokio::spawn(async move {
        let file: Option<File>;
        let dir = tauri::api::path::download_dir()
            .unwrap()
            .join(status_obj.lock().await.get_item().get_file_name());
        if dir.as_path().exists() && status_obj.lock().await.get_size() < size {
            file = Some(
                std::fs::OpenOptions::new()
                    .append(true)
                    .write(true)
                    .open(dir.as_path())
                    .unwrap(),
            );
        } else if dir.as_path().exists() && status_obj.lock().await.get_size() == size {
            status_obj.lock().await.get_item().concat_number();
            file = Some(std::fs::File::create(dir.as_path()).unwrap());
        } else {
            file = Some(std::fs::File::create(dir.as_path()).unwrap());
        }
        let mut stream = res.bytes_stream();
        let mut new_size = status_obj.lock().await.get_size();
        while let Some(b) = stream.next().await {
            new_size += b.as_ref().unwrap().len() as u64;
            let update = serde_json::to_string(&DownloadInfo::new(id, new_size)).unwrap();
            h.emit_all("ondownloadupdate", update).unwrap();
            file.as_ref().unwrap().write_all(&b.unwrap()).unwrap();
            status_obj.lock().await.set_curr_size(new_size);
            if status_obj.lock().await.is_paused() {
                drop(file);
                return Err::<(), u32>(status_obj.lock().await.get_item().get_id());
            }
        }
        let curr = status_obj.lock().await.get_size();
        if curr == size {
            status_obj.lock().await.set_finished();
            let s = handle.state::<TauriState>();
            if s.downloads.lock().await.len() == 0 {
                s.downloads.lock().await.pop();
            } else {
                s.downloads
                    .lock()
                    .await
                    .remove(status_obj.lock().await.get_item().get_id() as usize);
            }
            drop(file);
            println!("Download Finished!");
        }
        Ok::<(), u32>(())
    })
    .await
    .unwrap()
    .unwrap_or_else(|err| {
        println!("Download with id {err} paused!");
    });
    Ok(())
}

async fn push_download(download: &Arc<Mutex<DownloadStatus>>, handle: AppHandle) {
    let tauri_state: tauri::State<TauriState> = handle.state();
    tauri_state.downloads.lock().await.push(download.clone());
    handle
        .emit_all(
            "ondownload",
            serde_json::to_string(&download.lock().await.get_item()).unwrap(),
        )
        .expect("Message Could not be emitted.");
}

fn remove_finished_downloads(handle: AppHandle) {
    let tauri_state: tauri::State<TauriState> = handle.state();
    let mut vec = tauri_state
        .downloads
        .try_lock()
        .expect("Could not acquire mutex lock!");
    vec.retain(|e| !e.try_lock().unwrap().is_finished());
}

#[post("/")]
async fn post_download(dw: String, data: web::Data<AppState>) -> std::io::Result<impl Responder> {
    let new_data = serde_json::from_str::<DownloadObj>(&dw).unwrap();
    // Get State
    let download = Arc::new(Mutex::new(DownloadStatus::new(new_data)));
    push_download(&download, data.handle.clone()).await;
    remove_finished_downloads(data.handle.clone());
    Ok(HttpResponse::Ok())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let h = app.handle();
            tauri::async_runtime::spawn(
                HttpServer::new(move || {
                    App::new()
                        .service(post_download)
                        .app_data(Data::new(AppState { handle: h.clone() }))
                })
                .bind(("localhost", 4000))?
                .run(),
            );
            Ok(())
        })
        .manage(TauriState {
            downloads: Arc::new(Mutex::new(Vec::<Arc<Mutex<DownloadStatus>>>::new())),
        })
        .invoke_handler(tauri::generate_handler![pause_download, resume, download])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
