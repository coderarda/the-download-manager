// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use actix_web::{
    post,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use futures_util::StreamExt;
use reqwest::header::{CONTENT_LENGTH, RANGE};
use std::{cmp::Ordering, fs::File, io::Write, sync::Arc, time::Duration};
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
async fn get_download_info(url: String) -> Result<DownloadObj, String> {
    let client = reqwest::Client::new();
    let res = client.head(url.clone()).send().await;
    match res {
        Ok(result) => {
            let head = result.headers();
            let len: u64 = head
                .get(CONTENT_LENGTH)
                .unwrap()
                .to_str()
                .unwrap()
                .parse()
                .unwrap();
            let filename = url.split("/").last().unwrap().to_string();
            return Ok(DownloadObj::new(0, url, filename, len));
        }
        Err(e) => {
            println!("reqwest Error!, {e}");
            Err(String::from("Error occured!"))
        }
    }
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
    for d in state.get_downloads().lock().await.iter_mut() {
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
            d.lock().await.set_item(download);
            tokio::spawn(download_with_pause(d.clone(), handle.clone()));
            break;
        }
    }
    Ok(())
}


// Change this function to download_url_with_pause and add another function without pause capabilities.
async fn download_with_pause(
    status: Arc<Mutex<DownloadStatus>>,
    handle: tauri::AppHandle,
) -> Result<(), reqwest::Error> {
    println!("Download starting...");
    let h = handle.clone();
    let id = status.lock().await.get_item().get_id();
    status.lock().await.set_downloading();
    let arg1 = status.lock().await.get_curr_size();
    let size = status.lock().await.get_item().get_total_size();
    let client = reqwest::Client::new();
    let res = client
        .get(status.lock().await.get_item().get_url())
        .header(RANGE, format!("bytes={arg1}-{size}"))
        .send()
        .await?;

    // Check if the server accepts range requests
    tokio::spawn(async move {
        let mut file: Option<File>;
        let mut dir = tauri::api::path::download_dir()
            .unwrap()
            .join(status.lock().await.get_item().get_file_name());
        if res
            .headers()
            .get("accept-ranges")
            .map_or(false, |v| v == "bytes")
        {
            match arg1.cmp(&size) {
                Ordering::Less => {
                    if status.lock().await.get_curr_size() == 0 {
                        file = Some(std::fs::File::create(dir.as_path()).unwrap());
                    } else {
                        file = Some(
                            std::fs::OpenOptions::new()
                                .append(true)
                                .open(dir.as_path())
                                .unwrap(),
                        );
                    }
                }
                Ordering::Equal => {
                    status.lock().await.get_item().concat_number();
                    dir.pop();
                    dir.push(status.lock().await.get_item().get_file_name());
                    file = Some(std::fs::File::create(dir.as_path()).unwrap());
                }
                Ordering::Greater => {
                    panic!("Anomaly occurred! Canceling...");
                    // Send event to frontend to delete file and try again
                }
            }

            let mut stream = res.bytes_stream();
            let mut new_size = status.lock().await.get_curr_size();
            while let Some(b) = stream.next().await {
                if status.lock().await.is_paused() {
                    return Err::<(), u8>(status.lock().await.get_item().get_id());
                }
                match b {
                    Ok(chunk) => {
                        new_size += chunk.len() as u64;
                        let update =
                            serde_json::to_string(&DownloadInfo::new(id, new_size)).unwrap();
                        println!("{}", update);
                        h.emit_all("ondownloadupdate", update).unwrap();
                        match file {
                            Some(ref mut f) => {
                                f.write_all(&chunk).unwrap();
                                status.lock().await.set_curr_size(new_size);
                            }
                            None => {
                                println!("File not open!");
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error reading stream: {:?}", e);
                        return Err::<(), u8>(status.lock().await.get_item().get_id());
                    }
                }
            }
        }
        let curr = status.lock().await.get_curr_size();
        if curr == size {
            status.lock().await.set_finished();
            let s = handle.state::<AppDownloadManager>();
            if s.get_downloads().lock().await.len() != 0 {
                s.get_downloads()
                    .lock()
                    .await
                    .remove(status.lock().await.get_item().get_id() as usize);
            }
        }
        Ok(())
    })
    .await
    .unwrap()
    .unwrap_or_else(|e| {
        println!("Download with id {e} has been paused.");
    });

    Ok(())
}

async fn push_download_to_vec(download: &Arc<Mutex<DownloadStatus>>, handle: AppHandle) {
    let tauri_state: tauri::State<AppDownloadManager> = handle.state();
    tauri_state.get_downloads().lock().await.push(download.clone());
    handle
        .emit_all(
            "ondownload",
            serde_json::to_string(&download.lock().await.get_item()).unwrap(),
        )
        .expect("Message Could not be emitted.");
}

fn remove_finished_downloads(handle: AppHandle) {
    tokio::spawn(async move {
        let state: tauri::State<AppDownloadManager> = handle.state();
        let vec = state.get_downloads();
        vec.lock().await.retain(|e| !e.try_lock().unwrap().is_finished());
    });
}

#[post("/")]
async fn listen_for_downloads(dw: String, data: web::Data<WebServerState>) -> std::io::Result<impl Responder> {
    let new_data = serde_json::from_str::<DownloadObj>(&dw).unwrap();
    // Get State
    let download = Arc::new(Mutex::new(DownloadStatus::new(new_data)));
    push_download_to_vec(&download, data.handle.clone()).await;
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
                        .service(listen_for_downloads)
                        .app_data(Data::new(WebServerState { handle: h.clone() }))
                })
                .bind(("localhost", 4000))?
                .run(),
            );
            Ok(())
        })
        .manage(AppDownloadManager::new(Arc::new(Mutex::new(Vec::<Arc<Mutex<DownloadStatus>>>::new()))))
        .invoke_handler(tauri::generate_handler![
            pause_download,
            resume,
            download,
            get_download_info
        ])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
