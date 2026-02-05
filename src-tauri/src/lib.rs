use reqwest::header::CONTENT_DISPOSITION;
use reqwest::header::RANGE;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tauri::Emitter;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;
#[tauri::command]
async fn download_file(
    app: tauri::AppHandle,
    url: String,
    filepath: String,
    threads: u64,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| e.to_string())?;
    // 1. Get Size
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let total_size = response.content_length().unwrap_or(0);
    if total_size < 1 {
        return Err("File has no size!".to_string());
    }
    // CRITICAL: Tell Frontend the size immediately (Fixes 0GB bug)
    let _ = app.emit("download-start", total_size);
    println!("Starting Download: {}", url);
    println!(
        "File Size: {} bytes ({} MB)",
        total_size,
        total_size / 1024 / 1024
    );
    let start_time = std::time::Instant::now();
    // 2. Allocator (Sparse File)
    let path = PathBuf::from(&filepath);
    {
        let mut file = File::create(&path).map_err(|e| e.to_string())?;
        file.seek(SeekFrom::Start(total_size - 1))
            .map_err(|e| e.to_string())?;
        file.write_all(&[0]).map_err(|e| e.to_string())?;
    }
    // 3. Dynamic Chunking (Tiered Optimization)
    let chunk_size = if total_size < 100 * 1024 * 1024 {
        512 * 1024 // 512KB for < 100MB
    } else if total_size < 1 * 1024 * 1024 * 1024 {
        4 * 1024 * 1024 // 4MB for 100MB - 1GB
    } else if total_size < 10 * 1024 * 1024 * 1024 {
        16 * 1024 * 1024 // 16MB for 1GB - 10GB
    } else {
        64 * 1024 * 1024 // 64MB for > 10GB
    };
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
    let actual_threads = if threads > 0 { threads } else { 8 };
    let mut handles = vec![];
    println!(
        "Starting {} threads (Dynamic Mode) | Chunk Size: {} MB",
        actual_threads,
        chunk_size / 1024 / 1024
    );
    for _ in 0..actual_threads {
        let url = url.clone();
        let path = path.clone();
        let app_handle = app.clone();
        let queue = chunk_queue.clone();
        let retry_counts = chunk_retry_counts.clone();
        let completed = completed_chunks.clone();
        let downloaded_counter = downloaded_bytes.clone();
        let stats_monitor = speed_stats.clone();
        let handle = tokio::spawn(async move {
            let client = reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .read_timeout(std::time::Duration::from_secs(5)) // Aggressive 5s timeout
                .connect_timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap();
            let file = tokio::fs::OpenOptions::new()
                .write(true)
                .open(&path)
                .await
                .unwrap();
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
                let enforce_speed = retry_count < 3;
                if retry_count > 1 {
                    println!("Chunk {} retry attempt #{}", idx, retry_count);
                }

                let start = idx * chunk_size;
                let mut end = start + chunk_size - 1;
                if end >= total_size {
                    end = total_size - 1;
                }
                let range_header = format!("bytes={}-{}", start, end);
                let mut success = false;
                let mut attempts = 0;
                while attempts < 5 && !success {
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
                                        if elapsed > 3.0 {
                                            let speed =
                                                (bytes_this_attempt as f64 / 1024.0) / elapsed;
                                            if speed < 300.0 {
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

                                    println!(
                                        "✓ Chunk {} complete ({}/{}) - {} MB total",
                                        idx,
                                        new_completed,
                                        total_chunks,
                                        new / 1048576
                                    );

                                    // Emit progress update
                                    let _ = app_handle.emit("download-progress", new);
                                } else if chunk_ok {
                                    // Partial download - not actually complete!
                                    println!(
                                        "⚠ Chunk {} incomplete: {} / {} bytes",
                                        idx, bytes_this_attempt, expected_bytes
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
                    println!(
                        "✗ Chunk {} failed after 5 attempts! Pushing back to queue.",
                        idx
                    );
                    let mut q = queue.lock().await;
                    q.push_back(idx);
                }
            }
            writer.flush().await.unwrap();
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.await.map_err(|e| e.to_string())?;
    }
    let elapsed = start_time.elapsed();
    let avg_speed = (total_size as f64 / 1024.0 / 1024.0) / elapsed.as_secs_f64();
    println!("--- Download Summary ---");
    println!("Time: {:.2} seconds", elapsed.as_secs_f64());
    println!("Average Speed: {:.2} MB/s", avg_speed);
    {
        let monitor = speed_stats.lock().await;
        let (_, peak, min, _) = *monitor;
        println!("Peak Speed: {:.2} MB/s", peak);
        println!(
            "Min Speed: {:.2} MB/s",
            if min == f64::MAX { 0.0 } else { min }
        );
    }
    // INTEGRITY CHECK
    println!("Verifying integrity...");
    let final_bytes = downloaded_bytes.load(Ordering::SeqCst);
    let final_completed = completed_chunks.load(Ordering::SeqCst);

    println!("Completed chunks: {} / {}", final_completed, total_chunks);
    println!(
        "Downloaded bytes: {} / {} ({:.2}%)",
        final_bytes,
        total_size,
        (final_bytes as f64 / total_size as f64) * 100.0
    );

    if final_bytes < total_size {
        return Err(format!(
            "Download FAILED: {} / {} bytes ({} / {} chunks). Retry.",
            final_bytes, total_size, final_completed, total_chunks
        ));
    }

    println!("Integrity Check PASSED: 100%");
    let _ = app.emit("download-progress", total_size);
    Ok(format!("Done!"))
}
#[tauri::command]
async fn get_file_details(url: String) -> Result<(String, u64), String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| e.to_string())?;
    // 1. Try HEAD request first
    let mut response = client.head(&url).send().await;
    // 2. Fallback to GET if HEAD was rejected
    if response.is_err() || !response.as_ref().unwrap().status().is_success() {
        println!("HEAD failed, trying GET for details...");
        response = client.get(&url).header(RANGE, "bytes=0-0").send().await;
    }
    let response = response.map_err(|e| format!("Request failed: {}", e))?;
    if !response.status().is_success() {
        return Err(format!("Server returned error: {}", response.status()));
    }
    let size = response.content_length().unwrap_or(0);

    let mut filename = "download.dat".to_string();
    if let Some(disp) = response.headers().get(CONTENT_DISPOSITION) {
        if let Ok(disp_str) = disp.to_str() {
            if let Some(name_part) = disp_str.split("filename=").nth(1) {
                filename = name_part
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
            }
        }
    } else {
        if let Ok(parsed_url) = url::Url::parse(&url) {
            if let Some(segments) = parsed_url.path_segments() {
                if let Some(last) = segments.last() {
                    if !last.is_empty() {
                        filename = last.to_string();
                    }
                }
            }
        }
    }
    Ok((filename, size))
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![download_file, get_file_details])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
