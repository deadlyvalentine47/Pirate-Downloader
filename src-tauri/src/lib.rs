// Module declarations
mod commands;
mod core;
mod ipc;
mod network;
mod utils;

// Imports
use reqwest::header::RANGE;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Emitter;
use tracing::debug;

// Module imports
use core::error::DownloadError;
use core::{state, types};
use network::{client, headers};
use utils::{filesystem, logger};
// Import the struct from engine, do NOT redefine it
use core::engine::DownloadCommandResult;

// REMOVED DUPLICATE STRUCT DEFINITION

#[tauri::command]
async fn download_file(
    app: tauri::AppHandle,
    url: String,
    filepath: String,
    threads: u64,
    manager: tauri::State<'_, commands::DownloadManager>,
) -> Result<DownloadCommandResult, DownloadError> {
    let path = PathBuf::from(&filepath);
    let target_dir = path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();
    let filename_hint = path.file_name().map(|s| s.to_string_lossy().to_string());

    start_download(
        app,
        url,
        target_dir,
        filename_hint,
        threads,
        (*manager).clone(),
    )
    .await
}

/// Shared entry point for starting a download (used by Command and IPC)
/// Shared entry point for starting a download (used by Command and IPC)
pub async fn start_download(
    app: tauri::AppHandle,
    url: String,
    target_dir: PathBuf,
    filename_hint: Option<String>,
    threads: u64,
    manager: commands::DownloadManager,
) -> Result<DownloadCommandResult, DownloadError> {
    // Generate unique download ID
    let download_id = uuid::Uuid::new_v4().to_string();
    let download_control = Arc::new(commands::DownloadControl::new());
    let client = client::create_client()?;

    // 1. Get Response & Size
    let response = client.get(&url).send().await?;
    let total_size = response.content_length().unwrap_or(0);

    // Resolve Filename
    let final_filename = if let Some(hint) = filename_hint {
        sanitize_filename::sanitize(hint)
    } else {
        headers::extract_filename(&response, &url)
    };

    let filepath = target_dir.join(&final_filename);
    let filepath_str = filepath.to_string_lossy().to_string();

    if total_size < 1 {
        // Warning log, but maybe proceed?
        // For sparse file allocation we NEED size.
        // MVP3 decision: Fail if no size, as per existing logic.
        return Err(DownloadError::Config(
            "File has no size! Server did not report Content-Length.".to_string(),
        ));
    }

    // CRITICAL: Tell Frontend the size AND download ID immediately
    let _ = app.emit("download-start", total_size);
    let _ = app.emit("download-id", download_id.clone());

    tracing::info!(download_id = %download_id, filename = %final_filename, size_mb = total_size / (1024 * 1024), "Starting download");

    // 2. Allocator
    filesystem::allocate_sparse_file(&filepath, total_size)?;

    // 3. Register
    let actual_threads = if threads > 0 {
        threads
    } else {
        types::DEFAULT_THREADS
    } as u32;
    let chunk_size = filesystem::calculate_chunk_size(total_size);
    let total_chunks = (total_size + chunk_size - 1) / chunk_size;

    let metadata = state::DownloadMetadata {
        url: url.clone(),
        filepath: filepath_str,
        total_size,
        downloaded_bytes: 0,
        state: state::DownloadState::Active,
        thread_count: actual_threads,
        completed_chunks: vec![],
        incomplete_chunks: (0..total_chunks).collect(),
        created_at: chrono::Utc::now(),
        paused_at: None,
        resumed_at: None,
        stopped_at: None,
        completed_at: None,
        error_message: None,
    };

    manager
        .register_download(
            download_id.clone(),
            metadata.clone(),
            download_control.clone(),
        )
        .await;

    // 4. Run Loop (Delegated to Engine)
    core::engine::DownloadEngine::start(app, download_id, metadata, download_control, manager, 0)
        .await
}

#[tauri::command]
async fn get_file_details(url: String) -> Result<(String, u64), DownloadError> {
    fetch_file_details(&url).await
}

/// Fetches real filename and size from the server
pub async fn fetch_file_details(url: &str) -> Result<(String, u64), DownloadError> {
    let client = client::create_client()?;

    // 1. Try HEAD request first
    let mut response = client.head(url).send().await;

    // 2. Fallback to GET if HEAD was rejected or failed
    if response.is_err()
        || !response
            .as_ref()
            .ok()
            .map_or(false, |r| r.status().is_success())
    {
        debug!("HEAD request failed, falling back to GET with range header");
        response = client.get(url).header(RANGE, "bytes=0-0").send().await;
    }

    let response = response.map_err(|e| DownloadError::Network(e.to_string()))?;
    if !response.status().is_success() {
        return Err(DownloadError::Config(format!(
            "Server returned error: {}",
            response.status()
        )));
    }

    let size = response.content_length().unwrap_or(0);
    let filename = headers::extract_filename(&response, url);

    Ok((filename, size))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logger (console + file in dev, file only in prod)
    if let Err(e) = logger::init_logger() {
        eprintln!("Failed to initialize logger: {}", e);
    }

    // Initialize download manager
    let download_manager = commands::DownloadManager::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(download_manager)
        .setup(|app| {
            ipc::init(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            download_file,
            get_file_details,
            commands::download_control::pause_download,
            commands::download_control::resume_download,
            commands::download_control::stop_download,
            commands::download_control::cancel_download,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
