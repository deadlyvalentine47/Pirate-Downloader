use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use pirate_shared::{IpcMessage, IPC_NAME};
use std::io::{BufRead, BufReader};
use tauri::{AppHandle, Emitter, Manager};
use tracing::{error, info};

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
    // Attempt to get real filename/size before asking user
    // This makes the modal much more useful
    let (filename, size) = match crate::fetch_file_details(&req.url).await {
        Ok(details) => details,
        Err(e) => {
            info!(
                "Could not fetch details for URL {}: {}. Falling back to URL guessing.",
                req.url, e
            );
            let guessed_name = req
                .url
                .split('/')
                .last()
                .unwrap_or("download")
                .split('?')
                .next()
                .unwrap_or("download")
                .to_string();
            (guessed_name, 0)
        }
    };

    // Sanitize just in case
    let safe_filename = sanitize_filename::sanitize(filename);

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
            "size": size
        }),
    )?;

    // Bring App to Foreground
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.set_focus();
    }

    Ok(())
}
