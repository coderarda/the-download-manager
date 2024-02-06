// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use actix_web::{
    post,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use curl::easy::Easy2;
use std::{path::Path, sync::{Arc, Mutex}};
use tauri::{AppHandle, Manager};

mod download;
use download::{curl_handler::MyCurlHandler, DownloadInfo, DownloadObj, DownloadStatus};

mod tauri_state;
use tauri_state::TauriState;
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

#[derive(Clone)]
struct AppState {
    handle: AppHandle,
}

#[tauri::command]
fn pause_download(state: tauri::State<'_, TauriState>, id: u32) -> Result<(), String> {
    let mut vec = state.downloads.try_lock().expect("Could not acquire mutex lock!");
    for d in vec.iter_mut() {
        if d.lock().unwrap().get_item().get_id() == id {
            d.lock().unwrap().set_pause();
        }
    }  
    Ok(())
}

#[tauri::command]
fn resume(state: tauri::State<'_, TauriState>, id: u32) -> Result<(), String> {
    state.downloads.lock().unwrap().iter_mut().for_each(|d| {
        if d.lock().unwrap().get_item().get_id() == id {
            d.lock().unwrap().resume_download();
        }
    });
    Ok(())
}

fn download_self(status_obj: Arc<Mutex<DownloadStatus>>, handle: tauri::AppHandle) {
    // Fix communication and pause parts. 
    let file = std::fs::File::create(Path::new(
        tauri::api::path::download_dir()
            .unwrap()
            .join(status_obj.lock().unwrap().get_item().get_file_name())
            .as_path(),
    ))
    .unwrap();
    let obj = status_obj.lock().unwrap().get_item();
    let mut easy = Easy2::new(MyCurlHandler::new(Arc::clone(&status_obj), file, handle.clone()));
    easy.get(true).unwrap();
    easy.progress(true).unwrap();
    easy.url(obj.get_url().as_str()).unwrap();
    let h = handle.clone();
    let new_obj = status_obj.clone();
    let event_id = handle.listen_global("onbackendupdate", move |event| {
        let tmp = new_obj.lock().unwrap();
        let update = serde_json::to_string(&DownloadInfo::new(
            tmp.get_item().get_id(),
            event.payload().unwrap().parse::<u64>().unwrap(),
        ))
        .unwrap();
        h.emit_all("ondownloadupdate", update).unwrap();
    });

    std::thread::spawn(move || {
        status_obj.lock().unwrap().set_downloading();
        easy.perform().unwrap();
        let mut obj = status_obj.lock().unwrap();
        while obj.is_downloading() {
            if obj.will_resume() {
                easy.unpause_write().unwrap();
                obj.set_resume_false();
            }
        }
        handle.unlisten(event_id);
    }).join().unwrap();
}

fn push_download(download: &Arc<Mutex<DownloadStatus>>, handle: AppHandle) {
    let tauri_state: tauri::State<TauriState> = handle.state();
    handle
        .emit_all(
            "ondownload",
            serde_json::to_string(&download.lock().unwrap().get_item()).unwrap(),
        )
        .expect("Message Could not be emitted.");  
    tauri_state.downloads.lock().unwrap().push(download.clone()); 
    // Mutate inside vector...
    // Move the removal part here (inside thread)
}

fn remove_finished_downloads(handle: AppHandle) {
    let tauri_state: tauri::State<TauriState> = handle.state();
    let mut vec = tauri_state.downloads.try_lock().expect("Could not acquire mutex lock!");
    vec.retain(|d| !d.lock().unwrap().is_finished());
}

#[post("/")]
async fn post_download(dw: String, data: web::Data<AppState>) -> std::io::Result<impl Responder> {
    let new_data = serde_json::from_str::<DownloadObj>(&dw).unwrap();
    // Get State
    let download = Arc::new(Mutex::new(DownloadStatus::new(new_data)));
    push_download(&download, data.handle.clone());
    // Download not being removed?
    download_self(download, data.handle.clone());
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
        .invoke_handler(tauri::generate_handler![
            pause_download,
            resume,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
