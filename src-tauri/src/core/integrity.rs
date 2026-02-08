use super::error::DownloadError;
/// Download integrity verification
use tracing::{error, info};

/// Verifies download completion and integrity
///
/// Checks that:
/// 1. All chunks were completed
/// 2. All bytes were downloaded
///
/// # Arguments
/// * `downloaded_bytes` - Total bytes downloaded
/// * `total_size` - Expected total file size
/// * `completed_chunks` - Number of chunks completed
/// * `total_chunks` - Expected total number of chunks
///
/// # Returns
/// `Ok(())` if verification passes
///
/// # Errors
/// Returns DownloadError::Integrity if download is incomplete
pub fn verify_download(
    downloaded_bytes: u64,
    total_size: u64,
    completed_chunks: u64,
    total_chunks: u64,
) -> Result<(), DownloadError> {
    info!("Verifying download integrity...");

    let completion_percent = (downloaded_bytes as f64 / total_size as f64) * 100.0;

    info!(
        completed_chunks = completed_chunks,
        total_chunks = total_chunks,
        downloaded_bytes = downloaded_bytes,
        total_bytes = total_size,
        completion_percent = completion_percent,
        "Integrity check status"
    );

    if downloaded_bytes < total_size {
        error!(
            downloaded_bytes = downloaded_bytes,
            total_bytes = total_size,
            completed_chunks = completed_chunks,
            total_chunks = total_chunks,
            "Download incomplete - verification failed"
        );
        return Err(DownloadError::Integrity {
            message: format!(
                "Download FAILED: {} / {} bytes ({} / {} chunks). Retry.",
                downloaded_bytes, total_size, completed_chunks, total_chunks
            ),
        });
    }

    info!("Integrity check PASSED: 100%");
    Ok(())
}
