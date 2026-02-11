use crate::commands::{self};
use crate::core::error::DownloadError;
use crate::core::{integrity, state, types};
use crate::network::client;
use crate::utils;
use reqwest::header::RANGE;
use std::io::SeekFrom;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::Emitter;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error, info, warn};

/// Result returned by download commands to the frontend
#[derive(serde::Serialize)]
pub struct DownloadCommandResult {
    pub id: String,
    pub status: String, // "completed", "paused", "stopped"
}

pub struct DownloadEngine;

impl DownloadEngine {
    /// Core download loop - shared by new downloads and resumes
    pub async fn start(
        app: tauri::AppHandle,
        download_id: String,
        metadata: state::DownloadMetadata,
        control: Arc<commands::DownloadControl>,
        manager: commands::DownloadManager,
        generation: u32,
    ) -> Result<DownloadCommandResult, DownloadError> {
        let url = metadata.url.clone();
        let filepath = metadata.filepath.clone();
        let total_size = metadata.total_size;
        let actual_threads = metadata.thread_count as usize;

        let chunk_size = utils::filesystem::calculate_chunk_size(total_size);
        let total_chunks = (total_size + chunk_size - 1) / chunk_size;

        // Use shared control structures
        let downloaded_bytes = control.downloaded_bytes.clone();
        let completed_chunks = control.completed_chunks.clone();

        // Initialize queue from incomplete chunks
        // Crucial for resume support
        let chunk_queue = Arc::new(tokio::sync::Mutex::new(std::collections::VecDeque::from(
            metadata.incomplete_chunks.clone(),
        )));

        // Track retries
        let chunk_retry_counts = Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::<
            u64,
            u32,
        >::new()));

        // Speed stats
        let speed_stats = Arc::new(tokio::sync::Mutex::new((
            0u64,
            0.0f64,
            f64::MAX,
            std::time::Instant::now(),
        )));

        info!(
            download_id = %download_id,
            threads = actual_threads,
            chunk_size_mb = chunk_size / 1024 / 1024,
            remaining_chunks = metadata.incomplete_chunks.len(),
            "Starting download workers"
        );

        let start_time = std::time::Instant::now();
        let mut handles = vec![];

        for _ in 0..actual_threads {
            let url = url.clone();
            let path = filepath.clone();
            let app_handle = app.clone();
            let queue = chunk_queue.clone();
            let retry_counts = chunk_retry_counts.clone();
            let _stats_monitor = speed_stats.clone();
            let control = control.clone();
            let worker_downloaded = downloaded_bytes.clone();
            let worker_completed = completed_chunks.clone();

            let handle = tokio::spawn(async move {
                // Worker implementation
                let client = client::create_worker_client();
                let path_buf = PathBuf::from(path);

                let file = match tokio::fs::OpenOptions::new()
                    .write(true)
                    .open(&path_buf)
                    .await
                {
                    Ok(f) => f,
                    Err(e) => {
                        error!(error = %e, "Failed to open file for writing");
                        return;
                    }
                };
                let mut writer = tokio::io::BufWriter::with_capacity(128 * 1024, file);

                loop {
                    // 1. Check Signals
                    let signal = control.signal.load(Ordering::Relaxed);
                    if signal != 0 {
                        debug!("Worker received signal {}, exiting", signal);
                        break;
                    }

                    // 2. Get Job
                    let idx_opt = {
                        let mut q = queue.lock().await;
                        q.pop_front()
                    };

                    if idx_opt.is_none() {
                        // Check if actually done
                        let completed_count = worker_completed.lock().await.len() as u64;
                        if completed_count >= total_chunks {
                            break;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        continue;
                    }
                    let idx = idx_opt.unwrap();

                    // Check retry count
                    let retry_count = {
                        let mut counts = retry_counts.lock().await;
                        let count = counts.entry(idx).or_insert(0);
                        *count += 1;
                        *count
                    };

                    // Speed limit for struggling chunks
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
                        attempts += 1;
                        // Check signal again inside retry loop
                        if control.signal.load(Ordering::Relaxed) != 0 {
                            break;
                        }

                        // CRITICAL: Check generation to kill zombie tasks
                        if control.generation.load(Ordering::Relaxed) != generation {
                            debug!("Worker generation mismatch, exiting");
                            break;
                        }

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
                                        // CRITICAL: Check signal inside streaming loop
                                        if control.signal.load(Ordering::Relaxed) != 0 {
                                            chunk_ok = false;
                                            break;
                                        }

                                        // Speed enforcement logic
                                        if enforce_speed {
                                            let elapsed = attempt_start.elapsed().as_secs_f64();
                                            if elapsed > types::SPEED_ENFORCEMENT_DELAY {
                                                let speed =
                                                    (bytes_this_attempt as f64 / 1024.0) / elapsed;
                                                if speed < types::SPEED_ENFORCEMENT_THRESHOLD {
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
                                    }

                                    let expected_bytes = (end - start + 1) as u64;
                                    if chunk_ok && bytes_this_attempt == expected_bytes {
                                        success = true;

                                        // Update shared state
                                        let old_bytes = worker_downloaded
                                            .fetch_add(expected_bytes, Ordering::Relaxed);
                                        let new_bytes = old_bytes + expected_bytes;

                                        worker_completed.lock().await.push(idx);
                                        let completed_count = worker_completed.lock().await.len();

                                        debug!(
                                            chunk_id = idx,
                                            completed = completed_count,
                                            total = total_chunks,
                                            mb_total = new_bytes / 1048576,
                                            "Chunk complete"
                                        );

                                        let _ = app_handle.emit("download-progress", new_bytes);
                                    }
                                }
                            }
                        }

                        if !success {
                            // CRITICAL: Check signal before sleeping
                            if control.signal.load(Ordering::Relaxed) != 0 {
                                break;
                            }
                            tokio::time::sleep(std::time::Duration::from_millis(200 * attempts))
                                .await;
                        }
                    } // End retry loop

                    if !success {
                        // Push back to queue
                        if control.signal.load(Ordering::Relaxed) == 0 {
                            error!(chunk_id = idx, "Chunk failed after max attempts");
                            let mut q = queue.lock().await;
                            q.push_back(idx);
                        }
                    }
                } // End main loop

                let _ = writer.flush().await;
            });
            handles.push(handle);
        }

        // Spawn Monitor Task to Sync Manager State
        let monitor_manager = manager.clone();
        let monitor_id = download_id.clone();
        let monitor_control = control.clone();
        let monitor_handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                if monitor_control.signal.load(Ordering::Relaxed) != 0 {
                    break;
                }
                // Sync progress
                let bytes = monitor_control.downloaded_bytes.load(Ordering::Relaxed);
                if let Some(mut meta) = monitor_manager.get_download(&monitor_id).await {
                    meta.downloaded_bytes = bytes;
                    // We don't sync completed_chunks vector constantly to avoid lock contention
                    // But we should sync it on pause/stop in control logic
                    monitor_manager.update_download(&monitor_id, meta).await;
                }
            }
        });

        // Wait for workers
        for handle in handles {
            handle
                .await
                .map_err(|e| DownloadError::TaskJoin(e.to_string()))?;
        }

        // CRITICAL: Check if we finished due to a signal (Pause/Stop/Cancel)
        // If signaled, we MUST NOT run completion logic
        let final_signal = control.signal.load(Ordering::SeqCst);
        info!(
            "Download loop finished. Final signal check: {}",
            final_signal
        );

        if final_signal != 0 {
            info!("Download task finished due to signal: {}", final_signal);
            let status = match final_signal {
                1 => "paused",
                2 => "stopped",
                _ => "cancelled",
            };
            return Ok(DownloadCommandResult {
                id: download_id,
                status: status.to_string(),
            });
        }
        monitor_handle.abort();

        // Check final status
        let signal = control.signal.load(Ordering::Relaxed);
        if signal != 0 {
            // Stopped/Paused/Cancelled
            let status = match signal {
                1 => "paused",
                2 => "stopped",
                _ => "cancelled",
            };
            return Ok(DownloadCommandResult {
                id: download_id,
                status: status.to_string(),
            });
        }

        // If we are here, we should be done. Verify.
        let elapsed = start_time.elapsed();
        let avg_speed = (total_size as f64 / 1024.0 / 1024.0) / elapsed.as_secs_f64();
        let (_peak_speed, _min_speed) = {
            let monitor = speed_stats.lock().await;
            let (_, peak, min, _) = *monitor;
            (peak, if min == f64::MAX { 0.0 } else { min })
        };

        info!(
            duration_secs = elapsed.as_secs_f64(),
            avg_speed_mbps = avg_speed,
            "Download complete"
        );

        // Integrity Check
        let final_bytes = downloaded_bytes.load(Ordering::SeqCst);
        let final_completed_count = completed_chunks.lock().await.len() as u64;

        integrity::verify_download(final_bytes, total_size, final_completed_count, total_chunks)?;

        let _ = app.emit("download-progress", total_size);

        // Mark complete in manager
        if let Some(mut meta) = manager.get_download(&download_id).await {
            meta.complete();
            meta.downloaded_bytes = total_size;

            // Emit completion event
            let _ = app.emit("download-state", "completed");

            // Populate completed chunks fully for correctness
            // meta.completed_chunks = ...

            manager.update_download(&download_id, meta).await;
        }

        // Remove from manager?
        // Wait, if we remove it, we lose history?
        // User might want to see history.
        // Logic said "Cleanup: Remove download from manager".
        // If we keep history, we should keep it.
        // For now, keep inconsistent logic: remove from manager (active downloads), but maybe persistent state remains?
        manager.remove_download(&download_id).await;
        info!(download_id = %download_id, "Download completed and removed from manager");

        Ok(DownloadCommandResult {
            id: download_id,
            status: "completed".to_string(),
        })
    }
}
