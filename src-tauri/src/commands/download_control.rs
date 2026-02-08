// use crate::core::error::DownloadError;
use crate::core::persistence::{delete_state, save_state};
use crate::core::state::{DownloadMetadata, DownloadState};
use std::sync::Arc;
/// Download control commands module
///
/// This module contains all Tauri commands related to download control operations:
/// - pause_download: Pause an active download
/// - resume_download: Resume a paused/stopped download
/// - stop_download: Gracefully stop a download
/// - cancel_download: Cancel and cleanup a download
use tauri::{Emitter, State};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// Shared download state manager
/// This will be used to track active downloads and their state
#[derive(Clone)]
pub struct DownloadManager {
    /// Map of download ID to metadata
    active_downloads: Arc<Mutex<std::collections::HashMap<String, DownloadMetadata>>>,

    /// Map of download ID to control signals
    download_controls: Arc<Mutex<std::collections::HashMap<String, Arc<DownloadControl>>>>,
}

/// Control signals for active downloads
/// Used to communicate pause/stop/cancel commands to worker threads
#[derive(Clone)]
pub struct DownloadControl {
    /// Control signal: 0=run, 1=pause, 2=stop, 3=cancel
    pub signal: Arc<std::sync::atomic::AtomicU8>,

    /// Completed chunks (for state saving)
    pub completed_chunks: Arc<Mutex<Vec<u64>>>,

    /// Downloaded bytes counter
    pub downloaded_bytes: Arc<std::sync::atomic::AtomicU64>,

    /// Task generation ID (to invalidate old workers)
    pub generation: Arc<std::sync::atomic::AtomicU32>,
}

impl DownloadControl {
    pub fn new() -> Self {
        Self {
            signal: Arc::new(std::sync::atomic::AtomicU8::new(0)),
            completed_chunks: Arc::new(Mutex::new(Vec::new())),
            downloaded_bytes: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            generation: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        }
    }

    /// Check if download should continue
    pub fn should_continue(&self) -> bool {
        self.signal.load(std::sync::atomic::Ordering::Relaxed) == 0
    }

    /// Get current signal value
    pub fn get_signal(&self) -> u8 {
        self.signal.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            active_downloads: Arc::new(Mutex::new(std::collections::HashMap::new())),
            download_controls: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Register a new download with control signals
    pub async fn register_download(
        &self,
        id: String,
        metadata: DownloadMetadata,
        control: Arc<DownloadControl>,
    ) {
        let mut downloads = self.active_downloads.lock().await;
        downloads.insert(id.clone(), metadata);

        let mut controls = self.download_controls.lock().await;
        controls.insert(id, control);
    }

    /// Get download control signals
    pub async fn get_control(&self, id: &str) -> Option<Arc<DownloadControl>> {
        let controls = self.download_controls.lock().await;
        controls.get(id).cloned()
    }

    /// Get download metadata
    pub async fn get_download(&self, id: &str) -> Option<DownloadMetadata> {
        let downloads = self.active_downloads.lock().await;
        downloads.get(id).cloned()
    }

    /// Update download metadata
    pub async fn update_download(&self, id: &str, metadata: DownloadMetadata) {
        let mut downloads = self.active_downloads.lock().await;
        downloads.insert(id.to_string(), metadata);
    }

    /// Remove download from active list
    pub async fn remove_download(&self, id: &str) {
        let mut downloads = self.active_downloads.lock().await;
        downloads.remove(id);
    }
}

/// Pause an active download
///
/// Saves the current state to disk and signals the download to stop.
/// The download can be resumed later from the exact same position.
///
/// # Arguments
/// * `download_id` - Unique identifier for the download
///
/// # Returns
/// * `Ok(())` if pause was successful
/// * `Err(String)` if pause failed
#[tauri::command]
pub async fn pause_download(
    download_id: String,
    manager: State<'_, DownloadManager>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    info!(download_id = %download_id, "Pausing download");

    // Get current download metadata
    let mut metadata = manager.get_download(&download_id).await.ok_or_else(|| {
        error!(download_id = %download_id, "Download not found");
        format!("Download {} not found", download_id)
    })?;

    // Check if download can be paused
    if !metadata.state.is_active() {
        warn!(
            download_id = %download_id,
            current_state = ?metadata.state,
            "Cannot pause download - not in active state"
        );
        return Err(format!(
            "Cannot pause download in state: {:?}",
            metadata.state
        ));
    }

    // Update state to paused
    metadata.pause();

    // Get control signal to sync state
    if let Some(control) = manager.get_control(&download_id).await {
        // Sync downloaded bytes
        let bytes = control
            .downloaded_bytes
            .load(std::sync::atomic::Ordering::Relaxed);
        metadata.downloaded_bytes = bytes;

        // Sync completed chunks
        let completed = control.completed_chunks.lock().await.clone();
        metadata.completed_chunks = completed.clone();

        // Update incomplete chunks
        metadata
            .incomplete_chunks
            .retain(|id| !completed.contains(id));

        // Set signal to pause
        control
            .signal
            .store(1, std::sync::atomic::Ordering::Relaxed);
        debug!(download_id = %download_id, "Set pause signal and synced state");
    }

    // Save state to disk
    save_state(&metadata).map_err(|e| {
        error!(download_id = %download_id, error = %e, "Failed to save paused state");
        e.to_string()
    })?;

    // Update in-memory state
    manager
        .update_download(&download_id, metadata.clone())
        .await;

    info!(
        download_id = %download_id,
        downloaded_bytes = metadata.downloaded_bytes,
        progress = %format!("{:.2}%", metadata.progress_percentage()),
        "Download paused successfully"
    );

    // Emit state change event to frontend
    let _ = app.emit("download-state", "paused");

    Ok(())
}

/// Resume a paused or stopped download
///
/// Loads the saved state from disk and continues downloading from where it left off.
///
/// # Arguments
/// * `download_id` - Unique identifier for the download
///
/// # Returns
/// * `Ok(())` if resume was successful
/// * `Err(String)` if resume failed
#[tauri::command]
pub async fn resume_download(
    download_id: String,
    manager: State<'_, DownloadManager>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    info!(download_id = %download_id, "Resuming download");

    let mut metadata = manager.get_download(&download_id).await.ok_or_else(|| {
        error!(download_id = %download_id, "Download not found");
        format!("Download {} not found", download_id)
    })?;

    if !metadata.state.can_resume() {
        warn!(
            download_id = %download_id,
            current_state = ?metadata.state,
            "Cannot resume download - not in pausable state"
        );
        return Err(format!(
            "Cannot resume download in state: {:?}",
            metadata.state
        ));
    }

    metadata.resume();

    // Save state
    save_state(&metadata).map_err(|e| e.to_string())?;
    manager
        .update_download(&download_id, metadata.clone())
        .await;

    // Get control
    let control = manager
        .get_control(&download_id)
        .await
        .ok_or_else(|| "Control signals not found".to_string())?;

    // Reset signal
    control
        .signal
        .store(0, std::sync::atomic::Ordering::Relaxed);

    // Initialize control state from metadata (CRITICAL for resume)
    control.downloaded_bytes.store(
        metadata.downloaded_bytes,
        std::sync::atomic::Ordering::SeqCst,
    );
    {
        let mut completed_lock = control.completed_chunks.lock().await;
        *completed_lock = metadata.completed_chunks.clone();
    }

    // Increment generation to invalidate old workers
    control
        .generation
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let generation = control.generation.load(std::sync::atomic::Ordering::SeqCst);

    info!(download_id = %download_id, generation = generation, "Download resumed, spawning background task");
    let _ = app.emit("download-state", "active");

    // Spawn background task
    let app_handle = app.clone();
    let app_handle_error = app.clone();
    let manager_cloned = (*manager).clone();
    let id_cloned = download_id.clone();
    let meta_cloned = metadata.clone();
    let control_cloned = control.clone();

    tokio::spawn(async move {
        match crate::run_download_task(
            app_handle,
            id_cloned.clone(),
            meta_cloned,
            control_cloned,
            manager_cloned,
            generation,
        )
        .await
        {
            Ok(res) => {
                info!(download_id = %id_cloned, status = %res.status, "Resume task completed")
            }
            Err(e) => {
                error!(download_id = %id_cloned, error = %e, "Resume task failed");
                let _ = app_handle_error.emit("download-state", "failed");
            }
        }
    });

    Ok(())
}

/// Stop a download gracefully
///
/// Saves the current state and stops the download. The partial file is kept on disk.
/// The download can be resumed later.
///
/// # Arguments
/// * `download_id` - Unique identifier for the download
///
/// # Returns
/// * `Ok(())` if stop was successful
/// * `Err(String)` if stop failed
#[tauri::command]
pub async fn stop_download(
    download_id: String,
    manager: State<'_, DownloadManager>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    info!(download_id = %download_id, "Stopping download");

    // Get current download metadata
    let mut metadata = manager.get_download(&download_id).await.ok_or_else(|| {
        error!(download_id = %download_id, "Download not found");
        format!("Download {} not found", download_id)
    })?;

    // Update state to stopped
    metadata.stop();

    // Get control signal to sync state
    if let Some(control) = manager.get_control(&download_id).await {
        // Sync downloaded bytes
        let bytes = control
            .downloaded_bytes
            .load(std::sync::atomic::Ordering::Relaxed);
        metadata.downloaded_bytes = bytes;

        // Sync completed chunks
        let completed = control.completed_chunks.lock().await.clone();
        metadata.completed_chunks = completed.clone();

        // Update incomplete chunks
        metadata
            .incomplete_chunks
            .retain(|id| !completed.contains(id));

        // Set control signal to stop (2)
        control
            .signal
            .store(2, std::sync::atomic::Ordering::Relaxed);
        debug!(download_id = %download_id, "Set stop signal and synced state");
    }

    // Save state to disk
    save_state(&metadata).map_err(|e| {
        error!(download_id = %download_id, error = %e, "Failed to save stopped state");
        e.to_string()
    })?;

    // Update in-memory state
    manager
        .update_download(&download_id, metadata.clone())
        .await;

    info!(
        download_id = %download_id,
        downloaded_bytes = metadata.downloaded_bytes,
        progress = %format!("{:.2}%", metadata.progress_percentage()),
        "Download stopped successfully"
    );

    // Emit state change event to frontend
    let _ = app.emit("download-state", "stopped");

    Ok(())
}

/// Cancel a download and cleanup all files
///
/// Terminates the download immediately, deletes the partial file and state file.
/// This operation cannot be undone.
///
/// # Arguments
/// * `download_id` - Unique identifier for the download
///
/// # Returns
/// * `Ok(())` if cancel was successful
/// * `Err(String)` if cancel failed
#[tauri::command]
pub async fn cancel_download(
    download_id: String,
    manager: State<'_, DownloadManager>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    warn!(download_id = %download_id, "Cancelling download - will cleanup all files");

    // Get current download metadata
    let mut metadata = manager.get_download(&download_id).await.ok_or_else(|| {
        error!(download_id = %download_id, "Download not found");
        format!("Download {} not found", download_id)
    })?;

    let filepath = metadata.filepath.clone();

    // Update state to cancelled
    metadata.cancel();

    // Set control signal to cancel (3)
    if let Some(control) = manager.get_control(&download_id).await {
        control
            .signal
            .store(3, std::sync::atomic::Ordering::Relaxed);
        debug!(download_id = %download_id, "Set cancel signal for download workers");
    }

    // Delete state file
    delete_state(&filepath).map_err(|e| {
        error!(download_id = %download_id, error = %e, "Failed to delete state file");
        e.to_string()
    })?;

    // Delete partial file if it exists
    if std::path::Path::new(&filepath).exists() {
        std::fs::remove_file(&filepath).map_err(|e| {
            error!(
                download_id = %download_id,
                filepath = %filepath,
                error = %e,
                "Failed to delete partial file"
            );
            e.to_string()
        })?;
        debug!(filepath = %filepath, "Deleted partial file");
    }

    // Remove from active downloads
    manager.remove_download(&download_id).await;

    info!(
        download_id = %download_id,
        "Download cancelled and cleaned up successfully"
    );

    // Emit state change event to frontend
    let _ = app.emit("download-state", "cancelled");

    Ok(())
}
