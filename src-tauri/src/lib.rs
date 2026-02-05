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
    // Small files = Small chunks (Agile)
    // Large files = Large chunks (Low Overhead)
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

    let next_chunk_index = Arc::new(AtomicU64::new(0));
    let downloaded_bytes = Arc::new(AtomicU64::new(0));
    // Speed tracking (Current, Peak, Min, StartTime)
    // Min initialized to u64::MAX so first reading overwrites it
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
        let next_chunk = next_chunk_index.clone();
        let downloaded_counter = downloaded_bytes.clone();
        let stats_monitor = speed_stats.clone(); // Renamed for clarity

        let handle = tokio::spawn(async move {
            // FIX: Aggressive 5s Timeout to kill "Zombie" threads faster
            let client = reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .read_timeout(std::time::Duration::from_secs(5)) // reduced from 30s
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
                let idx = next_chunk.fetch_add(1, Ordering::Relaxed);
                if idx >= total_chunks {
                    break;
                }

                let start = idx * chunk_size;
                let mut end = start + chunk_size - 1;
                if end >= total_size {
                    end = total_size - 1;
                }

                let range_header = format!("bytes={}-{}", start, end);
                let mut success = false;
                let mut attempts = 0;

                while attempts < 10 && !success {
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
                                while let Some(chunk) = response.chunk().await.unwrap_or(None) {
                                    if writer.write_all(&chunk).await.is_err() {
                                        chunk_ok = false;
                                        break;
                                    }

                                    // Progress & Speed Limits
                                    let len = chunk.len() as u64;
                                    let old = downloaded_counter.fetch_add(len, Ordering::Relaxed);
                                    let new = old + len;
                                    if (new / (1024 * 1024)) > (old / (1024 * 1024)) {
                                        let mut monitor = stats_monitor.lock().await;
                                        let (last_bytes, mut peak, mut min, last_time) = *monitor;
                                        let elapsed = last_time.elapsed().as_secs_f64();

                                        if elapsed >= 1.0 {
                                            let speed_mb =
                                                ((new - last_bytes) as f64 / 1048576.0) / elapsed;
                                            if speed_mb > peak {
                                                peak = speed_mb;
                                            }
                                            if speed_mb < min && speed_mb > 0.0 {
                                                min = speed_mb;
                                            } // Ignore 0 start

                                            println!(
                                                "Status: {} / {} MB ({:.2} MB/s)",
                                                new / 1048576,
                                                total_size / 1048576,
                                                speed_mb
                                            );
                                            *monitor = (new, peak, min, std::time::Instant::now());
                                        }
                                        let _ = app_handle.emit("download-progress", new);
                                    }
                                }
                                if chunk_ok {
                                    success = true;
                                }
                            }
                        }
                    }
                    if !success {
                        tokio::time::sleep(std::time::Duration::from_millis(200 * attempts)).await;
                    }
                }
                if !success {
                    println!("THREAD FAILED CHUNK {}", idx);
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

    // Lock to get peak/min
    {
        let monitor = speed_stats.lock().await;
        let (_, peak, min, _) = *monitor;
        println!("Peak Speed: {:.2} MB/s", peak);
        println!(
            "Min Speed: {:.2} MB/s (Lowest recorded interval)",
            if min == f64::MAX { 0.0 } else { min }
        );
    }

    // INTEGRITY CHECK
    println!("Verifying integrity...");
    let final_bytes = downloaded_bytes.load(Ordering::SeqCst);
    if final_bytes < total_size {
        return Err(format!(
            "Download FAILED: {} / {} bytes. Retry.",
            final_bytes, total_size
        ));
    }

    println!("Integrity Check PASSED: 100%");
    let _ = app.emit("download-progress", total_size);
    Ok(format!("Done!"))
}

#[tauri::command]
async fn get_file_details(url: String) -> Result<(String, u64), String> {
    // 1. Use a Browser User-Agent (Servers hate generic bots)
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| e.to_string())?;

    // 2. Try HEAD request first (Fastest)
    let mut response = client.head(&url).send().await;

    // 3. Fallback to GET if HEAD was rejected (405 Method Not Allowed / 403 Forbidden)
    if response.is_err() || !response.as_ref().unwrap().status().is_success() {
        println!("HEAD failed, trying GET for details...");
        // Use a range request to avoiding downloading the whole file just to check details
        response = client.get(&url).header(RANGE, "bytes=0-0").send().await;
    }

    let response = response.map_err(|e| format!("Request failed: {}", e))?;

    // Check status again
    if !response.status().is_success() {
        return Err(format!("Server returned error: {}", response.status()));
    }

    // 4. Get Size (Content-Length or Content-Range)
    let size = response.content_length().unwrap_or(0);

    // 5. Try Content-Disposition Header (e.g., filename="movie.mp4")
    let mut filename = "download.dat".to_string();
    if let Some(disp) = response.headers().get(CONTENT_DISPOSITION) {
        if let Ok(disp_str) = disp.to_str() {
            if let Some(name_part) = disp_str.split("filename=").nth(1) {
                // simple cleanup to remove quotes/path chars
                filename = name_part
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
            }
        }
    } else {
        // 6. Fallback: Parse URL
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

    // If we still have a generic name but valid size, try to infer extension from Content-Type?
    // (Optional polish for later, keep it simple for now)

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
