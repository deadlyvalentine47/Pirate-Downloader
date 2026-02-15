use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use pirate_shared::{IpcMessage, IPC_NAME};
use std::io::{BufRead, BufReader};
use tauri::{AppHandle, Emitter, Manager};
use tracing::{error, info, warn};
use crate::core::state::DownloadState;

pub fn init(app: AppHandle) {
    std::thread::spawn(move || {
        let name = IPC_NAME;

        // Remove existing socket on Unix/Mac
        #[cfg(not(windows))]
        if std::fs::metadata(name).is_ok() {
            let _ = std::fs::remove_file(name);
        }

        let listener = match LocalSocketListener::bind(name) {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind IPC socket {}: {}", name, e);
                return;
            }
        };

        info!("IPC Server listening on {}", name);

        for stream in listener.incoming() {
            match stream {
                Ok(conn) => {
                    let app_handle = app.clone();
                    std::thread::spawn(move || handle_connection(conn, app_handle));
                }
                Err(e) => error!("Incoming IPC connection failed: {}", e),
            }
        }
    });
}

fn handle_connection(conn: LocalSocketStream, app: AppHandle) {
    let reader = BufReader::new(conn);

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                error!("Error reading IPC line: {}", e);
                break;
            }
        };

        if line.is_empty() {
            continue;
        }

        match serde_json::from_str::<IpcMessage>(&line) {
            Ok(msg) => {
                info!("Received IPC Message: {:?}", msg);
                match msg {
                    IpcMessage::DownloadRequest(req) => {
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(e) = handle_download_request(app_handle, req).await {
                                error!("Failed to start download from IPC: {}", e);
                            }
                        });
                    }
                    IpcMessage::LinkUpdate(req) => {
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(e) = handle_link_update(app_handle, req).await {
                                error!("Failed to update link from IPC: {}", e);
                            }
                        });
                    }
                    IpcMessage::Ping => {}
                }
            }
            Err(e) => error!("Failed to deserialize IPC message: {}", e),
        }
    }
}

async fn handle_download_request(
    app: AppHandle,
    req: pirate_shared::DownloadRequest,
) -> anyhow::Result<()> {
    // 1. Check if we already have a good filename from the extension
    let ext_filename = req.filename.as_ref()
        .map(|f| f.trim())
        .filter(|f| !f.is_empty() && !f.contains("aW5kZXgu") && !f.contains("index.m3u8"));

    // 2. Attempt to get real size (and filename if ext didn't provide a good one)
    // We pass the referrer and headers to avoid 403 Forbidden
    let (fetched_filename, size) = match crate::fetch_file_details_with_headers(&req.url, &req.headers, req.referrer.as_deref()).await {
        Ok(details) => details,
        Err(e) => {
            info!(
                "Could not fetch details for URL {}: {}. Falling back to metadata.",
                req.url, e
            );
            (ext_filename.map(|s| s.to_string()).unwrap_or_else(|| "download".to_string()), 0u64)
        }
    };

    // 3. Final decision: Use extension name if it looks like a real title, otherwise use fetched name
    let final_filename = ext_filename.map(|s| s.to_string()).unwrap_or(fetched_filename);
    
    // Sanitize
    let safe_filename = sanitize_filename::sanitize(final_filename);

    info!(
        "Emitting download request confirmation for {} (size: {} bytes)",
        safe_filename, size
    );

    // Emit event to Frontend
    app.emit(
        "request-download-confirmation",
        serde_json::json!({
            "url": req.url,
            "filename": safe_filename,
            "size": size,
            "headers": req.headers,
            "referrer": req.referrer
        }),
    )?;

    // Bring App to Foreground
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.set_focus();
    }

    Ok(())
}

async fn handle_link_update(
    app: AppHandle,
    req: pirate_shared::DownloadRequest,
) -> anyhow::Result<()> {
    let manager = app.state::<crate::commands::DownloadManager>();
    
    // Find a download that is in WaitingForLink state
    let mut target_id: Option<String> = None;
    let downloads = manager.get_all_downloads().await;
    
    for (id, meta) in downloads {
        if meta.state == DownloadState::WaitingForLink {
            // Found it! (Assuming one waiting for link for now)
            target_id = Some(id);
            break;
        }
    }

    if let Some(id) = target_id {
        info!("Updating link for download {}: {}", id, req.url);
        
        if let Some(mut meta) = manager.get_download(&id).await {
            meta.url = req.url.clone();
            // Automatically resume or wait for user?
            // IDM usually resumes automatically once link is refreshed.
            meta.state = DownloadState::Paused;
            manager.update_download(&id, meta).await;
            
            app.emit("download-link-updated", serde_json::json!({
                "id": id,
                "url": req.url
            }))?;
        }
    } else {
        warn!("Received LinkUpdate but no download is in WaitingForLink state.");
    }

    Ok(())
}
