// Module declarations
mod core;
mod network;
mod utils;

// Imports
use reqwest::header::RANGE;
use std::io::SeekFrom;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tauri::Emitter;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error, info, warn};

// Module imports
use core::error::DownloadError;
use core::integrity;
use core::types;
use network::{client, headers};
use utils::{filesystem, logger};

#[tauri::command]
async fn download_file(
    app: tauri::AppHandle,
    url: String,
    filepath: String,
    threads: u64,
) -> Result<String, DownloadError> {
    let client = client::create_client()?;

    // 1. Get Size
    let response = client.get(&url).send().await?;
    let total_size = response.content_length().unwrap_or(0);
    if total_size < 1 {
        return Err(DownloadError::Config("File has no size!".to_string()));
    }
    // CRITICAL: Tell Frontend the size immediately (Fixes 0GB bug)
    let _ = app.emit("download-start", total_size);
    info!(
        url = %url,
        size_bytes = total_size,
        size_mb = total_size / 1024 / 1024,
        "Starting download"
    );
    let start_time = std::time::Instant::now();

    // 2. Allocator (Sparse File)
    let path = PathBuf::from(&filepath);
    filesystem::allocate_sparse_file(&path, total_size)?;

    // 3. Dynamic Chunking (Tiered Optimization)
    let chunk_size = filesystem::calculate_chunk_size(total_size);
    let total_chunks = (total_size + chunk_size - 1) / chunk_size;
    // Robust Queue: If a chunk fails, it goes back here.
    let mut chunks_vec = Vec::new();
    for i in 0..total_chunks {
        chunks_vec.push(i);
    }
    let chunk_queue = Arc::new(tokio::sync::Mutex::new(std::collections::VecDeque::from(
        chunks_vec,
    )));
    // Track how many times each chunk has been attempted (to relax speed limits for struggling chunks)
    let chunk_retry_counts = Arc::new(tokio::sync::Mutex::new(
        std::collections::HashMap::<u64, u32>::new(),
    ));

    let completed_chunks = Arc::new(AtomicU64::new(0)); // Track successful completions
    let downloaded_bytes = Arc::new(AtomicU64::new(0));
    // Speed tracking (Current, Peak, Min, StartTime)
    let speed_stats = Arc::new(tokio::sync::Mutex::new((
        0u64,
        0.0f64,
        f64::MAX,
        std::time::Instant::now(),
    )));
    let actual_threads = if threads > 0 {
        threads
    } else {
        types::DEFAULT_THREADS
    };
    let mut handles = vec![];
    info!(
        threads = actual_threads,
        chunk_size_mb = chunk_size / 1024 / 1024,
        total_chunks = total_chunks,
        "Starting download workers"
    );
    for _ in 0..actual_threads {
        let url = url.clone();
        let path = path.clone();
        let app_handle = app.clone();
        let queue = chunk_queue.clone();
        let retry_counts = chunk_retry_counts.clone();
        let completed = completed_chunks.clone();
        let downloaded_counter = downloaded_bytes.clone();
        let _stats_monitor = speed_stats.clone();
        let handle = tokio::spawn(async move {
            let client = client::create_worker_client();

            let file = match tokio::fs::OpenOptions::new().write(true).open(&path).await {
                Ok(f) => f,
                Err(e) => {
                    error!(error = %e, "Failed to open file for writing");
                    return;
                }
            };
            let mut writer = tokio::io::BufWriter::with_capacity(128 * 1024, file);
            loop {
                // Check if all chunks are done
                let current_completed = completed.load(Ordering::Relaxed);
                if current_completed >= total_chunks {
                    break;
                }

                // Get Job
                let idx_opt = {
                    let mut q = queue.lock().await;
                    q.pop_front()
                };

                if idx_opt.is_none() {
                    // Queue empty but not all chunks done - wait and retry
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    continue;
                }
                let idx = idx_opt.unwrap();

                // Check retry count for this chunk
                let retry_count = {
                    let mut counts = retry_counts.lock().await;
                    let count = counts.entry(idx).or_insert(0);
                    *count += 1;
                    *count
                };

                // Disable speed enforcer for chunks that have been retried 3+ times
                let enforce_speed = retry_count < types::ADAPTIVE_RETRY_THRESHOLD;
                if retry_count > 1 {
                    warn!(
                        chunk_id = idx,
                        retry_count = retry_count,
                        "Chunk retry attempt"
                    );
                }

                let start = idx * chunk_size;
                let mut end = start + chunk_size - 1;
                if end >= total_size {
                    end = total_size - 1;
                }
                let range_header = format!("bytes={}-{}", start, end);
                let mut success = false;
                let mut attempts = 0;
                while attempts < types::CHUNK_RETRY_LIMIT && !success {
                    // 5 Retries per grab
                    attempts += 1;
                    if let Ok(mut response) = client
                        .get(&url)
                        .header(RANGE, range_header.clone())
                        .send()
                        .await
                    {
                        if response.status().is_success() {
                            if writer.seek(SeekFrom::Start(start)).await.is_ok() {
                                let mut chunk_ok = true;
                                let mut bytes_this_attempt = 0u64;
                                let attempt_start = std::time::Instant::now();
                                while let Some(chunk) = response.chunk().await.unwrap_or(None) {
                                    // SPEED ENFORCEMENT: > 300KB/s (but only for fresh chunks)
                                    if enforce_speed {
                                        let elapsed = attempt_start.elapsed().as_secs_f64();
                                        if elapsed > types::SPEED_ENFORCEMENT_DELAY {
                                            let speed =
                                                (bytes_this_attempt as f64 / 1024.0) / elapsed;
                                            if speed < types::SPEED_ENFORCEMENT_THRESHOLD {
                                                // Too slow? Kill it.
                                                chunk_ok = false;
                                                break;
                                            }
                                        }
                                    }
                                    if writer.write_all(&chunk).await.is_err() {
                                        chunk_ok = false;
                                        break;
                                    }

                                    bytes_this_attempt += chunk.len() as u64;

                                    // Note: We don't count bytes here anymore - only on successful completion
                                    // This prevents double-counting on retries
                                }

                                // Verify we downloaded the FULL chunk
                                let expected_bytes = (end - start + 1) as u64;
                                if chunk_ok && bytes_this_attempt == expected_bytes {
                                    success = true;

                                    // Only count bytes when chunk is FULLY complete (prevents double-counting)
                                    let old = downloaded_counter
                                        .fetch_add(expected_bytes, Ordering::Relaxed);
                                    let new = old + expected_bytes;

                                    let new_completed =
                                        completed.fetch_add(1, Ordering::Relaxed) + 1;

                                    debug!(
                                        chunk_id = idx,
                                        completed = new_completed,
                                        total = total_chunks,
                                        mb_total = new / 1048576,
                                        "Chunk complete"
                                    );

                                    // Emit progress update
                                    let _ = app_handle.emit("download-progress", new);
                                } else if chunk_ok {
                                    // Partial download - not actually complete!
                                    warn!(
                                        chunk_id = idx,
                                        downloaded = bytes_this_attempt,
                                        expected = expected_bytes,
                                        "Chunk incomplete - forcing retry"
                                    );
                                    chunk_ok = false; // Force retry
                                }
                            }
                        }
                    }
                    if !success {
                        tokio::time::sleep(std::time::Duration::from_millis(200 * attempts)).await;
                    }
                }
                if !success {
                    // CRITICAL: Push back to queue to be retried by ANY thread
                    error!(
                        chunk_id = idx,
                        attempts = types::CHUNK_RETRY_LIMIT,
                        "Chunk failed after max attempts - pushing back to queue"
                    );
                    let mut q = queue.lock().await;
                    q.push_back(idx);
                }
            }
            if let Err(e) = writer.flush().await {
                error!(error = %e, "Failed to flush writer");
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        handle
            .await
            .map_err(|e| DownloadError::TaskJoin(e.to_string()))?;
    }
    let elapsed = start_time.elapsed();
    let avg_speed = (total_size as f64 / 1024.0 / 1024.0) / elapsed.as_secs_f64();

    let (peak_speed, min_speed) = {
        let monitor = speed_stats.lock().await;
        let (_, peak, min, _) = *monitor;
        (peak, if min == f64::MAX { 0.0 } else { min })
    };

    info!(
        duration_secs = elapsed.as_secs_f64(),
        avg_speed_mbps = avg_speed,
        peak_speed_mbps = peak_speed,
        min_speed_mbps = min_speed,
        "Download complete"
    );

    // INTEGRITY CHECK
    let final_bytes = downloaded_bytes.load(Ordering::SeqCst);
    let final_completed = completed_chunks.load(Ordering::SeqCst);

    integrity::verify_download(final_bytes, total_size, final_completed, total_chunks)?;

    let _ = app.emit("download-progress", total_size);
    Ok(format!("Done!"))
}

#[tauri::command]
async fn get_file_details(url: String) -> Result<(String, u64), DownloadError> {
    let client = client::create_client()?;

    // 1. Try HEAD request first
    let mut response = client.head(&url).send().await;

    // 2. Fallback to GET if HEAD was rejected
    if response.is_err()
        || !response
            .as_ref()
            .ok()
            .map_or(false, |r| r.status().is_success())
    {
        debug!("HEAD request failed, falling back to GET with range header");
        response = client.get(&url).header(RANGE, "bytes=0-0").send().await;
    }

    let response = response.map_err(|e| DownloadError::Network(e.to_string()))?;
    if !response.status().is_success() {
        return Err(DownloadError::Config(format!(
            "Server returned error: {}",
            response.status()
        )));
    }

    let size = response.content_length().unwrap_or(0);
    let filename = headers::extract_filename(&response, &url);

    Ok((filename, size))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logger (console + file in dev, file only in prod)
    if let Err(e) = logger::init_logger() {
        eprintln!("Failed to initialize logger: {}", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![download_file, get_file_details])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
