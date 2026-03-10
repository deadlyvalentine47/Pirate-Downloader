use super::{DownloadContext, DownloadStrategy};
use crate::commands::DownloadCommandResult;
use crate::core::error::DownloadError;
use crate::core::{integrity, types};
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

pub struct HttpStrategy;

#[async_trait::async_trait]
impl DownloadStrategy for HttpStrategy {
    async fn execute(
        &self,
        context: &DownloadContext,
    ) -> Result<DownloadCommandResult, DownloadError> {
        let url = context.metadata.url.clone();
        let filepath = context.metadata.filepath.clone();
        let total_size = context.metadata.total_size;
        let actual_threads = context.metadata.thread_count as usize;

        let chunk_size = utils::filesystem::calculate_chunk_size(total_size);
        let total_chunks = (total_size + chunk_size - 1) / chunk_size;

        // Use shared control structures from context
        let downloaded_bytes = context.control.downloaded_bytes.clone();
        let completed_chunks = context.control.completed_chunks.clone();

        // Initialize queue from incomplete chunks
        let chunk_queue = Arc::new(tokio::sync::Mutex::new(std::collections::VecDeque::from(
            context.metadata.incomplete_chunks.clone(),
        )));

        let chunk_retry_counts = Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::<
            u64,
            u32,
        >::new()));
        
        let speed_stats = Arc::new(tokio::sync::Mutex::new((
            0u64,
            0.0f64,
            f64::MAX,
            std::time::Instant::now(),
        )));

        info!(
            download_id = %context.download_id,
            threads = actual_threads,
            chunk_size_mb = chunk_size / 1024 / 1024,
            remaining_chunks = context.metadata.incomplete_chunks.len(),
            "Starting download workers (HTTP Strategy)"
        );

        let start_time = std::time::Instant::now();
        let mut handles = vec![];

        let chunk_size_cloned = chunk_size;
        let total_size_cloned = total_size;
        let total_chunks_cloned = total_chunks;
        let manager_cloned = context.manager.clone();
        let download_id_cloned = context.download_id.clone();
        let headers_cloned = context.metadata.headers.clone();

        for _ in 0..actual_threads {
            let url = url.clone();
            let path = filepath.clone();
            let app_handle = context.app.clone();
            let queue = chunk_queue.clone();
            let retry_counts = chunk_retry_counts.clone();
            let _stats_monitor = speed_stats.clone();
            let control = context.control.clone();
            let worker_downloaded = downloaded_bytes.clone();
            let worker_completed = completed_chunks.clone();
            let generation = context.generation;
            
            let chunk_size = chunk_size_cloned;
            let total_size = total_size_cloned;
            let total_chunks = total_chunks_cloned;
            let manager = manager_cloned.clone();
            let download_id = download_id_cloned.clone();
            let headers = headers_cloned.clone();

            let handle = tokio::spawn(async move {
                let client = client::create_worker_client_with_headers(&headers);
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
                    let signal = control.signal.load(Ordering::Relaxed);
                    if signal != 0 {
                        debug!("Worker received signal {}, exiting", signal);
                        break;
                    }

                    let idx_opt = {
                        let mut q = queue.lock().await;
                        q.pop_front()
                    };

                    if idx_opt.is_none() {
                        let completed_count = worker_completed.lock().await.len() as u64;
                        if completed_count >= total_chunks {
                            break;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        continue;
                    }
                    let idx = idx_opt.unwrap();

                    let retry_count = {
                        let mut counts = retry_counts.lock().await;
                        let count = counts.entry(idx).or_insert(0);
                        *count += 1;
                        *count
                    };

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
                        if control.signal.load(Ordering::Relaxed) != 0 {
                            break;
                        }

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
                            if response.status() == reqwest::StatusCode::FORBIDDEN {
                                warn!(
                                    chunk_id = idx,
                                    "Received 403 Forbidden. Link likely expired. Triggering WaitingForLink state."
                                );
                                // Signal all workers to stop
                                control.signal.store(1, Ordering::SeqCst); // 1 = Paused/Stopped
                                
                                let app_handle_err = app_handle.clone();
                                let manager_err = manager.clone();
                                let download_id_err = download_id.clone();
                                
                                tokio::spawn(async move {
                                    if let Some(mut meta) = manager_err.get_download(&download_id_err).await {
                                        meta.wait_for_link();
                                        manager_err.update_download(&download_id_err, meta).await;
                                        let _ = app_handle_err.emit("download-state", "waitingforlink");
                                    }
                                });
                                break;
                            }

                            if response.status().is_success() {
                                if writer.seek(SeekFrom::Start(start)).await.is_ok() {
                                    let mut chunk_ok = true;
                                    let mut bytes_this_attempt = 0u64;
                                    let attempt_start = std::time::Instant::now();

                                    while let Some(chunk) = response.chunk().await.unwrap_or(None) {
                                        if control.signal.load(Ordering::Relaxed) != 0 {
                                            chunk_ok = false;
                                            break;
                                        }

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
                            if control.signal.load(Ordering::Relaxed) != 0 {
                                break;
                            }
                            tokio::time::sleep(std::time::Duration::from_millis(200 * attempts))
                                .await;
                        }
                    }

                    if !success {
                        if control.signal.load(Ordering::Relaxed) == 0 {
                            error!(chunk_id = idx, "Chunk failed after max attempts");
                            let mut q = queue.lock().await;
                            q.push_back(idx);
                        }
                    }
                }

                let _ = writer.flush().await;
            });
            handles.push(handle);
        }

        let monitor_manager = context.manager.clone();
        let monitor_id = context.download_id.clone();
        let monitor_control = context.control.clone();
        let monitor_app = context.app.clone();
        let monitor_total_size = total_size;
        
        let monitor_handle = tokio::spawn(async move {
            let mut last_bytes = monitor_control.downloaded_bytes.load(Ordering::Relaxed);
            let mut last_time = std::time::Instant::now();
            
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                
                if monitor_control.signal.load(Ordering::Relaxed) != 0 {
                    break;
                }
                
                let current_bytes = monitor_control.downloaded_bytes.load(Ordering::Relaxed);
                let now = std::time::Instant::now();
                let duration = now.duration_since(last_time).as_secs_f64();
                
                let bytes_diff = if current_bytes >= last_bytes {
                    current_bytes - last_bytes
                } else {
                    0
                };
                
                let speed = if duration > 0.0 {
                    (bytes_diff as f64 / duration) as u64
                } else {
                    0
                };
                
                let progress_pct = if monitor_total_size > 0 {
                    (current_bytes as f64 / monitor_total_size as f64) * 100.0
                } else {
                    0.0
                };
                
                let eta = if speed > 0 && monitor_total_size > current_bytes {
                    (monitor_total_size - current_bytes) / speed
                } else {
                    0
                };
                
                // Emit detailed progress
                let _ = monitor_app.emit("download-progress-detail", serde_json::json!({
                    "id": monitor_id,
                    "downloaded_bytes": current_bytes,
                    "total_bytes": monitor_total_size,
                    "progress_pct": progress_pct,
                    "speed": speed,
                    "eta": eta
                }));

                if let Some(mut meta) = monitor_manager.get_download(&monitor_id).await {
                    meta.downloaded_bytes = current_bytes;
                    monitor_manager.update_download(&monitor_id, meta).await;
                }
                
                last_bytes = current_bytes;
                last_time = now;
            }
        });

        for handle in handles {
            handle
                .await
                .map_err(|e| DownloadError::TaskJoin(e.to_string()))?;
        }

        let final_signal = context.control.signal.load(Ordering::SeqCst);
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
                id: context.download_id.clone(),
                status: status.to_string(),
            });
        }
        monitor_handle.abort();

        let elapsed = start_time.elapsed();
        let avg_speed = (total_size as f64 / 1024.0 / 1024.0) / elapsed.as_secs_f64();
        info!(
            duration_secs = elapsed.as_secs_f64(),
            avg_speed_mbps = avg_speed,
            "Download complete"
        );

        let final_bytes = downloaded_bytes.load(Ordering::SeqCst);
        let final_completed_count = completed_chunks.lock().await.len() as u64;

        integrity::verify_download(final_bytes, total_size, final_completed_count, total_chunks)?;

        let _ = context.app.emit("download-progress", total_size);

        if let Some(mut meta) = context.manager.get_download(&context.download_id).await {
            meta.complete();
            meta.downloaded_bytes = total_size;
            let _ = context.app.emit("download-state", "completed");
            context.manager.update_download(&context.download_id, meta).await;
        }

        context.manager.remove_download(&context.download_id).await;
        info!(download_id = %context.download_id, "Download completed and removed from manager");

        Ok(DownloadCommandResult {
            id: context.download_id.clone(),
            status: "completed".to_string(),
        })
    }
}
