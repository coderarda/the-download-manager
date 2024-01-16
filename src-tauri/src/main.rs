// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{path::Path, io::Write};

use actix_web::{
    post,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use tauri::{AppHandle, Manager};
mod download;
use download::{DownloadObj, DownloadInfo};
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

#[derive(Clone)]
struct AppState {
    handle: AppHandle,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[post("/")]
async fn post_download(dw: String, data: web::Data<AppState>) -> std::io::Result<impl Responder> {
    let new_data = serde_json::from_str::<DownloadObj>(&dw).unwrap();
    data.handle
        .emit_all("ondownload", serde_json::to_string(&new_data).unwrap())
        .expect("Message Could not be emitted.");

    let mut res = reqwest::get(new_data.get_url())
        .await
        .expect("No data exists at the URL.");

    let mut file = std::fs::File::create(
        Path::new(tauri::api::path::download_dir().unwrap().join(new_data.get_file_name()).as_path())
    ).unwrap();

    while let Some(buf) = res.chunk().await.expect("No chunk found!") {
        file.write(&buf).expect("Could not write to file!");
        let update = serde_json::to_string(&DownloadInfo::new(new_data.get_id(), buf.len() as u64)).unwrap();
        data.handle.emit_all("ondownloadupdate", update).unwrap();
    }

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
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
