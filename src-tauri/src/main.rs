// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use actix_web::{
    post,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use regex::Regex;
use reqwest::header::CONTENT_LENGTH;
use std::{io::Write, sync::Arc, time::Duration};
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
    let id = (state.get_downloads().lock().await.len()) as u8;
    let re = Regex::new(r"[^/]+$").unwrap();
    let title = re.find(&url)
        .map(|m| m.as_str().split('?').next().unwrap().to_string()).unwrap();
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
async fn download_manually_from_url(
    state: tauri::State<'_, AppDownloadManager>,
    handle: AppHandle,
    download: DownloadObj,
) -> Result<(), String> {
    let download_status = Arc::new(Mutex::new(DownloadStatus::new(download.clone())));
    state.get_downloads().lock().await.push(download_status.clone());
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
    download: DownloadObj,
) -> Result<(), String> {
    for d in state.get_downloads().lock().await.iter_mut() {
        if d.lock().await.get_item().get_id() == download.get_id() {
            tokio::spawn(download_item(d.clone(), handle.clone()));
            break;
        }
    }
    Ok(())
}

async fn download_item(
    status: Arc<Mutex<DownloadStatus>>,
    handle: tauri::AppHandle,
) -> Result<(), reqwest::Error> {
    println!("Download starting...");
    let filepath = tauri::api::path::download_dir()
        .unwrap()
        .join(status.lock().await.get_item().get_file_name());
    let h = handle.clone();
    let id = status.lock().await.get_item().get_id();
    status.lock().await.set_downloading();
    let mut file = std::fs::File::create(filepath).unwrap();
    let client = reqwest::Client::new();
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

        if count % 100 == 0 {
            let update = DownloadInfo::new(id, curr_sz);
            h.emit_all("ondownloadupdate", update).unwrap();
        }
        count += 1;
    }
    if curr_sz == status.lock().await.get_item().get_total_size() {
        let s = h.state::<AppDownloadManager>();
        if s.get_downloads().lock().await.len() != 0 {
            s.get_downloads()
                .lock()
                .await
                .remove(status.lock().await.get_item().get_id() as usize);
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
        .emit_all("ondownload", download.lock().await.get_item())
        .expect("Message Could not be emitted.");
}

#[post("/")]
async fn listen_for_downloads(
    dw: String,
    data: web::Data<WebServerState>,
) -> std::io::Result<impl Responder> {
    let new_data = serde_json::from_str::<DownloadObj>(&dw).unwrap();
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
        .invoke_handler(tauri::generate_handler![
            pause_download,
            resume,
            download,
            get_download_info,
            download_manually_from_url
        ])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
