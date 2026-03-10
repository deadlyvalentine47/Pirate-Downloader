use crate::core::error::DownloadError;
use super::processor::StreamProcessor;
use futures_util::StreamExt;
use reqwest::Client;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tracing::{debug, warn};
use std::collections::HashMap;
use tauri::Emitter;
use std::sync::atomic::Ordering;
use serde_json::json;

pub struct DownloadedSegment {
    pub index: usize,
    pub bytes: Vec<u8>,
}

pub struct ParallelDownloader {
    client: Arc<Client>,
    processor: Arc<StreamProcessor>,
    max_parallel: usize,
    high_water_mark: usize,
}

impl ParallelDownloader {
    pub fn new(
        client: Arc<Client>,
        processor: Arc<StreamProcessor>,
        max_parallel: usize,
        high_water_mark: usize,
    ) -> Self {
        Self {
            client,
            processor,
            max_parallel,
            high_water_mark,
        }
    }

    pub async fn download_segments(
        &self,
        segment_urls: Vec<String>,
        header_map: reqwest::header::HeaderMap,
        mut output_file: tokio::fs::File,
        app: tauri::AppHandle,
        download_id: String,
        control: Arc<crate::commands::DownloadControl>,
    ) -> Result<(), DownloadError> {
        let total_segments = segment_urls.len();
        let client = self.client.clone();
        let processor = self.processor.clone();
        let mut downloaded_segments = 0;
        
        let mut segment_stream = futures_util::stream::iter(segment_urls.into_iter().enumerate())
            .map(|(index, url)| {
                let client = client.clone();
                let headers = header_map.clone();
                let processor = processor.clone();
                async move {
                    let mut retry_count = 0;
                    while retry_count < 3 {
                        match client.get(&url).headers(headers.clone()).send().await {
                            Ok(resp) if resp.status().is_success() => {
                                match resp.bytes().await {
                                    Ok(bytes) => {
                                        // Clean the segment before returning it
                                        let cleaned_bytes = processor.clean_segment(bytes.to_vec());
                                        return Ok(DownloadedSegment { index, bytes: cleaned_bytes });
                                    }
                                    Err(e) => warn!("Failed to read bytes for segment {}: {}", index, e),
                                }
                            }
                            Ok(resp) => warn!("Server returned {} for segment {}", resp.status(), index),
                            Err(e) => warn!("Network error for segment {}: {}", index, e),
                        }
                        retry_count += 1;
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    }
                    Err(DownloadError::Network(format!("Failed to download segment {} after retries", index)))
                }
            })
            .buffered(self.max_parallel);

        let mut next_index_to_write = 0;
        let mut pending_segments: HashMap<usize, Vec<u8>> = HashMap::new();

        while let Some(result) = segment_stream.next().await {
            let segment = result?;
            
            // Progress Reporting
            downloaded_segments += 1;
            let current_segment_bytes = segment.bytes.len() as u64;
            
            if segment.index == next_index_to_write {
                output_file.write_all(&segment.bytes).await
                    .map_err(|e| DownloadError::Config(format!("Failed to write segment {}: {}", segment.index, e)))?;
                next_index_to_write += 1;

                while let Some(bytes) = pending_segments.remove(&next_index_to_write) {
                    output_file.write_all(&bytes).await
                        .map_err(|e| DownloadError::Config(format!("Failed to write segment {}: {}", next_index_to_write, e)))?;
                    next_index_to_write += 1;
                }
            } else {
                pending_segments.insert(segment.index, segment.bytes);
                if pending_segments.len() > self.high_water_mark {
                    debug!("Backpressure: Buffer full ({}), waiting for writer...", pending_segments.len());
                }
            }

            let current_total_bytes = control.downloaded_bytes.fetch_add(current_segment_bytes, Ordering::SeqCst) + current_segment_bytes;
            
            let progress_pct = (downloaded_segments as f64 / total_segments as f64) * 100.0;
            
            let _ = app.emit("download-progress-detail", json!({
                "id": download_id,
                "downloadedBytes": current_total_bytes,
                "totalBytes": 0, // Unknown for streaming usually
                "progress": progress_pct,
                "speed": 0,
                "eta": 0
            }));

            // Keep legacy emission
            let _ = app.emit("download-progress", current_total_bytes);
        }

        output_file.flush().await.map_err(|e| DownloadError::Config(e.to_string()))?;
        Ok(())
    }
}
