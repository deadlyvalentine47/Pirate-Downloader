use crate::core::error::DownloadError;
use crate::core::state::DownloadMetadata;
use std::fs;
/// State persistence - save and load download state to/from disk
///
/// This module handles serialization of download metadata to JSON files
/// and provides utilities for managing state files.
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};

/// Get the state file path for a given download file
///
/// State files are stored alongside the .part file with .part.state extension
/// Example: /downloads/file.zip.part -> /downloads/file.zip.part.state
pub fn get_state_file_path(filepath: &str) -> PathBuf {
    PathBuf::from(format!("{}.state", filepath))
}

/// Save download metadata to state file
///
/// # Arguments
/// * `metadata` - The download metadata to save
///
/// # Returns
/// * `Ok(())` if save was successful
/// * `Err(DownloadError)` if save failed
pub fn save_state(metadata: &DownloadMetadata) -> Result<(), DownloadError> {
    let state_path = get_state_file_path(&metadata.filepath);

    debug!(
        filepath = %metadata.filepath,
        state = ?metadata.state,
        downloaded_bytes = metadata.downloaded_bytes,
        total_size = metadata.total_size,
        "Saving download state"
    );

    // Serialize metadata to JSON
    let json = serde_json::to_string_pretty(metadata).map_err(|e| {
        error!(error = %e, "Failed to serialize download metadata");
        DownloadError::Serialization(e.to_string())
    })?;

    // Write to file
    fs::write(&state_path, json).map_err(|e| {
        error!(path = ?state_path, error = %e, "Failed to write state file");
        DownloadError::FileSystem(e.to_string())
    })?;

    info!(
        path = ?state_path,
        state = ?metadata.state,
        progress = %format!("{:.2}%", metadata.progress_percentage()),
        "Download state saved successfully"
    );

    Ok(())
}

/// Load download metadata from state file
///
/// # Arguments
/// * `filepath` - Path to the download file (.part file)
///
/// # Returns
/// * `Ok(DownloadMetadata)` if load was successful
/// * `Err(DownloadError)` if load failed or file doesn't exist
#[allow(dead_code)]
pub fn load_state(filepath: &str) -> Result<DownloadMetadata, DownloadError> {
    let state_path = get_state_file_path(filepath);

    debug!(path = ?state_path, "Loading download state");

    // Check if state file exists
    if !state_path.exists() {
        warn!(path = ?state_path, "State file does not exist");
        return Err(DownloadError::StateNotFound(filepath.to_string()));
    }

    // Read file contents
    let json = fs::read_to_string(&state_path).map_err(|e| {
        error!(path = ?state_path, error = %e, "Failed to read state file");
        DownloadError::FileSystem(e.to_string())
    })?;

    // Deserialize JSON
    let metadata: DownloadMetadata = serde_json::from_str(&json).map_err(|e| {
        error!(error = %e, "Failed to deserialize download metadata");
        DownloadError::Serialization(e.to_string())
    })?;

    info!(
        path = ?state_path,
        state = ?metadata.state,
        progress = %format!("{:.2}%", metadata.progress_percentage()),
        "Download state loaded successfully"
    );

    Ok(metadata)
}

/// Delete state file for a download
///
/// Used when cancelling a download or after successful completion
///
/// # Arguments
/// * `filepath` - Path to the download file (.part file)
pub fn delete_state(filepath: &str) -> Result<(), DownloadError> {
    let state_path = get_state_file_path(filepath);

    if !state_path.exists() {
        debug!(path = ?state_path, "State file does not exist, nothing to delete");
        return Ok(());
    }

    debug!(path = ?state_path, "Deleting state file");

    fs::remove_file(&state_path).map_err(|e| {
        error!(path = ?state_path, error = %e, "Failed to delete state file");
        DownloadError::FileSystem(e.to_string())
    })?;

    info!(path = ?state_path, "State file deleted successfully");

    Ok(())
}

/// Check if a state file exists for a download
#[allow(dead_code)]
pub fn state_exists(filepath: &str) -> bool {
    get_state_file_path(filepath).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::DownloadState;
    use tempfile::tempdir;

    #[test]
    fn test_state_file_path() {
        let path = get_state_file_path("/downloads/file.zip.part");
        assert_eq!(path, PathBuf::from("/downloads/file.zip.part.state"));
    }

    #[test]
    fn test_save_and_load_state() {
        let dir = tempdir().unwrap();
        let filepath = dir.path().join("test.zip.part");
        let filepath_str = filepath.to_str().unwrap();

        let mut metadata = DownloadMetadata::new(
            "https://example.com/test.zip".to_string(),
            filepath_str.to_string(),
            1024,
            16,
        );
        metadata.downloaded_bytes = 512;
        metadata.pause();

        // Save state
        save_state(&metadata).unwrap();

        // Verify state file exists
        assert!(state_exists(filepath_str));

        // Load state
        let loaded = load_state(filepath_str).unwrap();

        assert_eq!(loaded.url, metadata.url);
        assert_eq!(loaded.downloaded_bytes, 512);
        assert_eq!(loaded.state, DownloadState::Paused);
        assert!(loaded.paused_at.is_some());
    }

    #[test]
    fn test_delete_state() {
        let dir = tempdir().unwrap();
        let filepath = dir.path().join("test.zip.part");
        let filepath_str = filepath.to_str().unwrap();

        let metadata = DownloadMetadata::new(
            "https://example.com/test.zip".to_string(),
            filepath_str.to_string(),
            1024,
            16,
        );

        // Save and verify
        save_state(&metadata).unwrap();
        assert!(state_exists(filepath_str));

        // Delete and verify
        delete_state(filepath_str).unwrap();
        assert!(!state_exists(filepath_str));
    }
}
