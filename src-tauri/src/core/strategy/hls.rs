use super::{DownloadContext, DownloadStrategy};
use crate::commands::DownloadCommandResult;
use crate::core::error::DownloadError;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::Emitter;
use bytes::Bytes;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error, info, warn};
use url::Url;

pub struct HlsStrategy;

#[async_trait::async_trait]
impl DownloadStrategy for HlsStrategy {
    async fn execute(
        &self,
        context: &DownloadContext,
    ) -> Result<DownloadCommandResult, DownloadError> {
        let url = &context.metadata.url;
        let filepath = &context.metadata.filepath;

        info!(
            download_id = %context.download_id,
            url = %url,
            "Starting Native HLS download strategy (IDM Mode)"
        );

        // 1. Setup Client & Headers
        let client = crate::network::client::create_client()
            .map_err(|e| DownloadError::Network(e.to_string()))?;

        let mut header_map = reqwest::header::HeaderMap::new();
        
        // APPLY FULL DNA: Use every header captured from the browser
        for (k, v) in &context.metadata.headers {
            let key_lc = k.to_lowercase();
            // IDM Rule: Skip hop-by-hop headers that the OS network stack should handle
            if key_lc == "content-length" || key_lc == "host" || key_lc == "connection" || key_lc == "proxy-authorization" {
                continue;
            }
            if let (Ok(name), Ok(val)) = (
                reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                reqwest::header::HeaderValue::from_str(v),
            ) {
                header_map.insert(name, val);
            }
        }

        // Add Referer if not already present in DNA
        if let Some(ref_url) = &context.metadata.referrer {
            if !header_map.contains_key(reqwest::header::REFERER) {
                if let Ok(val) = reqwest::header::HeaderValue::from_str(ref_url) {
                    header_map.insert(reqwest::header::REFERER, val);
                }
            }
        }

        // 2. Fetch and Parse Manifest
        debug!(download_id = %context.download_id, "Fetching manifest with Full DNA headers");
        let manifest_resp = client
            .get(url)
            .headers(header_map.clone())
            .send()
            .await
            .map_err(|e| DownloadError::Network(format!("Failed to fetch manifest: {}", e)))?;

        let mut manifest_text = manifest_resp
            .text()
            .await
            .map_err(|e| DownloadError::Network(format!("Failed to read manifest: {}", e)))?;

        let mut base_url = Url::parse(url).map_err(|e| DownloadError::Config(e.to_string()))?;

        // 2.1 Master Playlist Resolution (Improved for better reliability)
        if manifest_text.contains("#EXT-X-STREAM-INF") {
            info!(download_id = %context.download_id, "Master playlist detected, selecting best quality");
            let mut best_url = None;
            let mut max_bandwidth = 0;
            
            let lines: Vec<&str> = manifest_text.lines().collect();
            for i in 0..lines.len() {
                let line = lines[i].trim();
                if line.starts_with("#EXT-X-STREAM-INF") {
                    let bandwidth = line.split("BANDWIDTH=")
                        .nth(1)
                        .and_then(|s| s.split(',').next())
                        .and_then(|s| s.split('\n').next()) // Just in case
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0);
                    
                    if i + 1 < lines.len() {
                        let variant_url = lines[i+1].trim();
                        if !variant_url.starts_with('#') && !variant_url.is_empty() {
                            if bandwidth >= max_bandwidth {
                                max_bandwidth = bandwidth;
                                best_url = Some(variant_url.to_string());
                            }
                        }
                    }
                }
            }

            if let Some(relative_url) = best_url {
                let final_variant_url = base_url.join(&relative_url).map_err(|e| DownloadError::Config(e.to_string()))?;
                info!(download_id = %context.download_id, bandwidth = max_bandwidth, url = %final_variant_url, "Switching to quality variant");
                
                let variant_resp = client
                    .get(final_variant_url.as_str())
                    .headers(header_map.clone())
                    .send()
                    .await
                    .map_err(|e| DownloadError::Network(format!("Failed to fetch variant manifest: {}", e)))?;
                
                manifest_text = variant_resp.text().await.map_err(|e| DownloadError::Network(e.to_string()))?;
                base_url = final_variant_url;
            }
        }

        let mut segment_urls = Vec::new();
        for line in manifest_text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let full_url = base_url
                .join(trimmed)
                .map_err(|e| DownloadError::Config(format!("Failed to resolve segment URL: {}", e)))?;
            segment_urls.push(full_url);
        }

        let total_segments = segment_urls.len();
        if total_segments == 0 {
            return Err(DownloadError::Config("No segments found in manifest".to_string()));
        }
        
        info!(download_id = %context.download_id, count = total_segments, "Manifest parsed successfully");

        // 3. Download and Append (Optimized Sliding Window Turbo Mode)
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(filepath)
            .await
            .map_err(|e| DownloadError::Config(format!("Failed to create output file: {}", e)))?;

        let (tx, mut rx) = tokio::sync::mpsc::channel::<(usize, Option<Bytes>)>(64);
        let client_arc = Arc::new(client);
        let header_map_arc = Arc::new(header_map);
        let semaphore = Arc::new(tokio::sync::Semaphore::new(10));
        
        // Spawn segments in chunks to prevent memory explosion
        let mut current_spawn_idx = 0;
        let window_size = 20; // Buffer 20 segments ahead
        
        let spawn_next = |start: usize, end: usize, tx: tokio::sync::mpsc::Sender<(usize, Option<Bytes>)>, client: Arc<reqwest::Client>, headers: Arc<reqwest::header::HeaderMap>, urls: Vec<Url>, sem: Arc<tokio::sync::Semaphore>| {
            for i in start..end {
                if i >= urls.len() { break; }
                let url = urls[i].clone();
                let tx_inner = tx.clone();
                let client_inner = client.clone();
                let headers_inner = headers.clone();
                let sem_inner = sem.clone();
                
                tokio::spawn(async move {
                    let _permit = sem_inner.acquire().await.unwrap();
                    let mut retry_count = 0;
                    let mut data = None;
                    while retry_count < 3 && data.is_none() {
                        match client_inner.get(url.as_str()).headers((*headers_inner).clone()).send().await {
                            Ok(resp) => {
                                if resp.status().is_success() {
                                    if let Ok(b) = resp.bytes().await {
                                        data = Some(b);
                                    }
                                } else {
                                    warn!("Segment {} failed with status: {}", i, resp.status());
                                }
                            }
                            Err(e) => warn!("Segment {} network error: {}", i, e),
                        }
                        if data.is_none() {
                            retry_count += 1;
                            tokio::time::sleep(std::time::Duration::from_millis(1000 * retry_count as u64)).await;
                        }
                    }
                    let _ = tx_inner.send((i, data)).await;
                });
            }
        };

        // Initial spawn
        let urls_cloned = segment_urls.clone();
        spawn_next(0, window_size, tx.clone(), client_arc.clone(), header_map_arc.clone(), urls_cloned.clone(), semaphore.clone());
        current_spawn_idx = window_size;

        // Collector & Sequential Writer
        let mut pending_segments: std::collections::HashMap<usize, Bytes> = std::collections::HashMap::new();
        let mut next_to_write = 0;
        let mut finished_count = 0;

        while next_to_write < total_segments {
            // Check for interruption
            if context.control.signal.load(Ordering::Relaxed) != 0 {
                info!(download_id = %context.download_id, "HLS download stopped by user");
                return Ok(DownloadCommandResult {
                    id: context.download_id.clone(),
                    status: "stopped".to_string(),
                });
            }

            tokio::select! {
                Some((index, data)) = rx.recv() => {
                    let bytes = data.ok_or_else(|| {
                        error!("Failed segment {} after all retries", index);
                        DownloadError::Network(format!("Failed segment {}", index))
                    })?;
                    pending_segments.insert(index, bytes);
                    
                    // Write sequential chunks
                    while let Some(bytes) = pending_segments.remove(&next_to_write) {
                        file.write_all(&bytes).await.map_err(|e| DownloadError::Config(e.to_string()))?;
                        next_to_write += 1;
                        finished_count += 1;
                        
                        // Spawn more as we write
                        if current_spawn_idx < total_segments {
                            spawn_next(current_spawn_idx, current_spawn_idx + 1, tx.clone(), client_arc.clone(), header_map_arc.clone(), urls_cloned.clone(), semaphore.clone());
                            current_spawn_idx += 1;
                        }

                        // Progress
                        if finished_count % 10 == 0 || finished_count == total_segments {
                            let _ = context.app.emit("download-progress", (finished_count * 100 / total_segments) as u64);
                            let _ = context.app.emit("download-progress-pulse", format!("Turbo Mode: {}/{} segments", finished_count, total_segments));
                        }
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {
                    return Err(DownloadError::Network("HLS download timed out".to_string()));
                }
            }
        }

        file.flush().await.map_err(|e| DownloadError::Config(e.to_string()))?;
        info!(download_id = %context.download_id, "HLS download complete");

        // Update manager state
        if let Some(mut meta) = context.manager.get_download(&context.download_id).await {
            meta.complete();
            let _ = context.app.emit("download-state", "completed");
            context.manager.update_download(&context.download_id, meta).await;
        }

        context.manager.remove_download(&context.download_id).await;

        Ok(DownloadCommandResult {
            id: context.download_id.clone(),
            status: "completed".to_string(),
        })
    }
}
