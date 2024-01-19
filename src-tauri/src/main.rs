// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::VecDeque;

use actix_web::{
    post,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
mod download;
use download::{DownloadObj, DownloadStatus};

use tauri_state::TauriState;
mod tauri_state;
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

#[derive(Clone)]
struct AppState {
    handle: AppHandle,
}

#[tauri::command]
async fn pause_download(state: tauri::State<'_, TauriState>, id: u32) -> Result<(), String> {
    for d in state.downloads.lock().unwrap().iter() {
        if d.item.get_id() == id {
            d.clone().set_pause();
        }
    }
    Ok(())
}

#[tauri::command]
async fn resume(state: tauri::State<'_, TauriState>, id: u32) -> Result<(), String> {
    for d in state.downloads.lock().unwrap().iter() {
        if d.item.get_id() == id {
            d.clone().resume_download();
        }
    }
    Ok(())
}

async fn flush_downloads(vec: &Mutex<VecDeque<DownloadStatus>>, handle: AppHandle) {
    let el = vec.lock().unwrap().pop_front().unwrap();
    handle.emit_all("ondownload", serde_json::to_string(&el.item).unwrap())
        .expect("Message Could not be emitted.");
    el.download_self(handle).await;
}

#[post("/")]
async fn post_download(dw: String, data: web::Data<AppState>) -> std::io::Result<impl Responder> {
    let new_data = serde_json::from_str::<DownloadObj>(&dw).unwrap();
    
    // Get State
    let tauri_state: tauri::State<TauriState> = data.handle.state();
    let mutex = &tauri_state.downloads;
    mutex.lock().unwrap().push_back(DownloadStatus::new(new_data));
    flush_downloads(mutex, data.handle.clone()).await;

    // Run retrieval part if pushed to vector etc.

    println!("File downloaded!");
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
        .manage(TauriState { downloads: Mutex::new(VecDeque::<DownloadStatus>::new()) })
        .invoke_handler(tauri::generate_handler![pause_download, resume])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
