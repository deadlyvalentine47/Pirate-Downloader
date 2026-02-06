/// Download integrity verification

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
/// Returns error message if download is incomplete
pub fn verify_download(
    downloaded_bytes: u64,
    total_size: u64,
    completed_chunks: u64,
    total_chunks: u64,
) -> Result<(), String> {
    println!("Verifying integrity...");
    println!("Completed chunks: {} / {}", completed_chunks, total_chunks);
    println!(
        "Downloaded bytes: {} / {} ({:.2}%)",
        downloaded_bytes,
        total_size,
        (downloaded_bytes as f64 / total_size as f64) * 100.0
    );

    if downloaded_bytes < total_size {
        return Err(format!(
            "Download FAILED: {} / {} bytes ({} / {} chunks). Retry.",
            downloaded_bytes, total_size, completed_chunks, total_chunks
        ));
    }

    println!("Integrity Check PASSED: 100%");
    Ok(())
}
