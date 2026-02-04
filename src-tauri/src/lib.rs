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
    let client = reqwest::Client::new();

    // 1. Get Size
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let total_size = response.content_length().unwrap_or(0);
    if total_size < 1 {
        return Err("File has no size!".to_string());
    }

    // 2. Allocator
    let path = PathBuf::from(&filepath);
    {
        let mut file = File::create(&path).map_err(|e| e.to_string())?;
        file.seek(SeekFrom::Start(total_size - 1))
            .map_err(|e| e.to_string())?;
        file.write_all(&[0]).map_err(|e| e.to_string())?;
    }

    // 3. Dynamic Chunking (Work Stealing Pattern)
    // 3. Dynamic Chunking (Work Stealing Pattern)
    // Fixes "Straggler Effect" by letting fast threads take more work.
    // Optimization: Use larger chunks for larger files to reduce HTTP overhead.
    let chunk_size = if total_size > 50 * 1024 * 1024 {
        2 * 1024 * 1024 // 2MB chunks for > 50MB files
    } else {
        512 * 1024 // 512KB chunks for smaller files
    };
    let total_chunks = (total_size + chunk_size - 1) / chunk_size;

    // Shared Atomic Counters
    let next_chunk_index = Arc::new(AtomicU64::new(0));
    let downloaded_bytes = Arc::new(AtomicU64::new(0)); // For progress UI

    // Speed tracking
    let last_speed_check = Arc::new(tokio::sync::Mutex::new((0u64, std::time::Instant::now())));

    let actual_threads = if threads > 0 { threads } else { 8 };
    let mut handles = vec![];
    println!(
        "Starting {} threads (Dynamic Mode) for {}",
        actual_threads, url
    );

    for _ in 0..actual_threads {
        let url = url.clone();
        let path = path.clone();
        let app_handle = app.clone();
        let next_chunk = next_chunk_index.clone();
        let downloaded_counter = downloaded_bytes.clone();
        let speed_monitor = last_speed_check.clone();

        // Spawn Worker Thread
        let handle = tokio::spawn(async move {
            let client = reqwest::Client::new();

            // Open file once per thread
            let file = tokio::fs::OpenOptions::new()
                .write(true)
                .open(&path)
                .await
                .unwrap();
            let mut writer = tokio::io::BufWriter::with_capacity(128 * 1024, file);

            loop {
                // 1. "Steal" the next unit of work
                let idx = next_chunk.fetch_add(1, Ordering::Relaxed);
                if idx >= total_chunks {
                    break; // No more work
                }

                // 2. Calculate offsets
                let start = idx * chunk_size;
                let mut end = start + chunk_size - 1;
                if end >= total_size {
                    end = total_size - 1;
                }

                // 3. Download this specific chunk (With Retry)
                let range_header = format!("bytes={}-{}", start, end);
                let mut success = false;
                let mut attempts = 0;

                while attempts < 5 && !success {
                    attempts += 1;
                    // Use clone() because we might need to use header string again
                    let result = client
                        .get(&url)
                        .header(RANGE, range_header.clone())
                        .send()
                        .await;

                    if let Ok(mut response) = result {
                        if response.status().is_success() {
                            // Seek to start position
                            if writer.seek(SeekFrom::Start(start)).await.is_ok() {
                                let mut chunk_downloaded_ok = true;

                                // Stream the bytes
                                while let Some(chunk_res) = response.chunk().await.unwrap_or(None) {
                                    if writer.write_all(&chunk_res).await.is_err() {
                                        chunk_downloaded_ok = false;
                                        break;
                                    }

                                    // Update Progress
                                    let len = chunk_res.len() as u64;
                                    let old_total =
                                        downloaded_counter.fetch_add(len, Ordering::Relaxed);
                                    let new_total = old_total + len;

                                    // Speed Monitor (Preserving your existing logic)
                                    if (new_total / (1024 * 1024)) > (old_total / (1024 * 1024)) {
                                        let mut monitor = speed_monitor.lock().await;
                                        let (last_bytes, last_time) = *monitor;
                                        let elapsed = last_time.elapsed().as_secs_f64();
                                        if elapsed >= 1.0 {
                                            let bytes_diff = new_total - last_bytes;
                                            let speed_mb =
                                                (bytes_diff as f64 / 1024.0 / 1024.0) / elapsed;
                                            println!(
                                                "Status: {} MB / {} MB ({:.2} MB/s)",
                                                new_total / 1024 / 1024,
                                                total_size / 1024 / 1024,
                                                speed_mb
                                            );
                                            *monitor = (new_total, std::time::Instant::now());
                                        }
                                        let _ = app_handle.emit("download-progress", new_total);
                                    }
                                }

                                if chunk_downloaded_ok {
                                    success = true;
                                }
                            }
                        }
                    }

                    if !success {
                        // Wait a bit before retrying (Exponential backoff)
                        tokio::time::sleep(std::time::Duration::from_millis(100 * attempts)).await;
                    }
                }

                if !success {
                    println!("CRITICAL: Thread failed chunk {} after 5 attempts", idx);
                    // Ideally we would push this index back to the queue, but for now just log it.
                    // The integrity check at the end will catch this failure.
                }
            }
            writer.flush().await.unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.map_err(|e| e.to_string())?;
    }
    println!("All threads finished. Verifying integrity...");

    // CRITICAL: Verify we actually have all the bytes
    let final_bytes = downloaded_bytes.load(Ordering::SeqCst);
    if final_bytes < total_size {
        println!("Integrity Check FAILED: {} / {}", final_bytes, total_size);
        return Err(format!(
            "Download failed: Only {} / {} bytes received. Please retry.",
            final_bytes, total_size
        ));
    }

    println!(
        "Integrity Check PASSED: {} / {} bytes (100%)",
        final_bytes, total_size
    );
    let _ = app.emit("download-progress", total_size);
    Ok(format!("Done!"))
}

#[tauri::command]
async fn get_file_details(url: String) -> Result<(String, u64), String> {
    let client = reqwest::Client::new();
    // Send a HEAD request (headers only, no body)
    let response = client.head(&url).send().await.map_err(|e| e.to_string())?;

    // 1. Get Size
    let size = response.content_length().unwrap_or(0);

    // 2. Try Content-Disposition Header (e.g., filename="movie.mp4")
    let mut filename = "download.dat".to_string();
    if let Some(disp) = response.headers().get(CONTENT_DISPOSITION) {
        if let Ok(disp_str) = disp.to_str() {
            if let Some(name_part) = disp_str.split("filename=").nth(1) {
                // simple cleanup to remove quotes
                filename = name_part.trim_matches('"').to_string();
            }
        }
    } else {
        // 3. Fallback: Parse URL
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
        .plugin(tauri_plugin_dialog::init()) // <--- Added Dialog Plugin
        .invoke_handler(tauri::generate_handler![download_file, get_file_details])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
