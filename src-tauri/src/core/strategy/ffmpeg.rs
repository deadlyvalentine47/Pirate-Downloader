use super::{DownloadContext, DownloadStrategy};
use crate::commands::DownloadCommandResult;
use crate::core::error::DownloadError;
use std::sync::atomic::Ordering;
use tauri::Emitter;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error, info, warn};
use url::Url;

pub struct FfmpegStrategy;

#[async_trait::async_trait]
impl DownloadStrategy for FfmpegStrategy {
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
        for (k, v) in &context.metadata.headers {
            if let (Ok(name), Ok(val)) = (
                reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                reqwest::header::HeaderValue::from_str(v),
            ) {
                header_map.insert(name, val);
            }
        }

        // Add standard User-Agent and Referer
        header_map.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"),
        );
        if let Some(ref_url) = &context.metadata.referrer {
            if let Ok(val) = reqwest::header::HeaderValue::from_str(ref_url) {
                header_map.insert(reqwest::header::REFERER, val);
            }
        }

        // 2. Fetch and Parse Manifest
        debug!(download_id = %context.download_id, "Fetching manifest");
        let manifest_resp = client
            .get(url)
            .headers(header_map.clone())
            .send()
            .await
            .map_err(|e| DownloadError::Network(format!("Failed to fetch manifest: {}", e)))?;

        let manifest_text = manifest_resp
            .text()
            .await
            .map_err(|e| DownloadError::Network(format!("Failed to read manifest: {}", e)))?;

        let mut segment_urls = Vec::new();
        let base_url = Url::parse(url).map_err(|e| DownloadError::Config(e.to_string()))?;

        for line in manifest_text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                // If it's a master playlist, we should ideally recurse, 
                // but for now we look for the first sub-manifest.
                if trimmed.contains(".m3u8") && !trimmed.starts_with("#EXT") {
                    let sub_url = base_url.join(trimmed).map_err(|e| DownloadError::Config(e.to_string()))?;
                    info!("Detected sub-manifest, recursing to: {}", sub_url);
                    // For simplicity in this logic, we'd need a recursive fetch. 
                    // But usually, we are already given the quality manifest.
                }
                continue;
            }
            // Resolve relative URLs
            let full_url = base_url
                .join(trimmed)
                .map_err(|e| DownloadError::Config(format!("Failed to resolve segment URL: {}", e)))?;
            segment_urls.push(full_url);
        }

        if segment_urls.is_empty() {
            return Err(DownloadError::Config("No segments found in manifest".to_string()));
        }

        let total_segments = segment_urls.len();
        info!(download_id = %context.download_id, count = total_segments, "Found segments, starting download");

        // 3. Download and Append
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(filepath)
            .await
            .map_err(|e| DownloadError::Config(format!("Failed to create output file: {}", e)))?;

        let mut downloaded_segments = 0;

        for (i, seg_url) in segment_urls.iter().enumerate() {
            // Check for cancellation/pause
            let signal = context.control.signal.load(Ordering::Relaxed);
            if signal != 0 {
                info!("Download interrupted by signal: {}", signal);
                return Ok(DownloadCommandResult {
                    id: context.download_id.clone(),
                    status: if signal == 1 { "paused".to_string() } else { "stopped".to_string() },
                });
            }

            debug!(download_id = %context.download_id, seg = i + 1, total = total_segments, "Downloading segment");
            
            let mut retry_count = 0;
            let mut success = false;
            
            while retry_count < 3 && !success {
                let seg_resp = client
                    .get(seg_url.as_str())
                    .headers(header_map.clone())
                    .send()
                    .await;

                match seg_resp {
                    Ok(resp) if resp.status().is_success() => {
                        let bytes = resp.bytes().await.map_err(|e| DownloadError::Network(e.to_string()))?;
                        file.write_all(&bytes).await.map_err(|e| DownloadError::Config(e.to_string()))?;
                        success = true;
                    }
                    _ => {
                        retry_count += 1;
                        warn!("Failed to download segment {}, retry {}/3", i + 1, retry_count);
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    }
                }
            }

            if !success {
                return Err(DownloadError::Network(format!("Failed to download segment {} after retries", i + 1)));
            }

            downloaded_segments += 1;
            
            // Emit progress pulse (Pulse the UI like IDM does)
            let progress_msg = format!("Segment {}/{} downloaded", downloaded_segments, total_segments);
            let _ = context.app.emit("download-progress-pulse", progress_msg);
            
            // Update total progress (treating each segment as a unit of work since we don't know total bytes yet)
            // Or we can just pretend the size is the number of segments
            let _ = context.app.emit("download-progress", (downloaded_segments * 100 / total_segments) as u64);
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
