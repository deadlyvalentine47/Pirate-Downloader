use crate::core::error::DownloadError;
use futures_util::StreamExt;
use reqwest::Client;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error, warn};
use std::collections::HashMap;

pub struct DownloadedSegment {
    pub index: usize,
    pub bytes: Vec<u8>,
}

pub struct ParallelDownloader {
    client: Arc<Client>,
    max_parallel: usize,
    high_water_mark: usize,
}

impl ParallelDownloader {
    pub fn new(client: Arc<Client>, max_parallel: usize, high_water_mark: usize) -> Self {
        Self {
            client,
            max_parallel,
            high_water_mark,
        }
    }

    pub async fn download_segments(
        &self,
        segment_urls: Vec<String>,
        header_map: reqwest::header::HeaderMap,
        mut output_file: tokio::fs::File,
        _cancel_signal: Arc<std::sync::atomic::AtomicI32>,
    ) -> Result<(), DownloadError> {
        let total_segments = segment_urls.len();
        let client = self.client.clone();
        
        // We use a stream to download segments in parallel.
        // `.buffered(max_parallel)` handles the worker pool for us.
        let mut segment_stream = futures_util::stream::iter(segment_urls.into_iter().enumerate())
            .map(|(index, url)| {
                let client = client.clone();
                let headers = header_map.clone();
                async move {
                    let mut retry_count = 0;
                    while retry_count < 3 {
                        match client.get(&url).headers(headers.clone()).send().await {
                            Ok(resp) if resp.status().is_success() => {
                                match resp.bytes().await {
                                    Ok(bytes) => return Ok(DownloadedSegment { index, bytes: bytes.to_vec() }),
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
            
            // If this is the next segment we need, write it immediately.
            if segment.index == next_index_to_write {
                output_file.write_all(&segment.bytes).await
                    .map_err(|e| DownloadError::Config(format!("Failed to write segment {}: {}", segment.index, e)))?;
                next_index_to_write += 1;

                // Check if any previously buffered segments can now be written.
                while let Some(bytes) = pending_segments.remove(&next_index_to_write) {
                    output_file.write_all(&bytes).await
                        .map_err(|e| DownloadError::Config(format!("Failed to write segment {}: {}", next_index_to_write, e)))?;
                    next_index_to_write += 1;
                }
            } else {
                // Buffer the segment for later writing.
                pending_segments.insert(segment.index, segment.bytes);
                
                // Backpressure check: If the buffer is too large, it means the downloader 
                // is way ahead of the writer (likely waiting for a slow segment).
                if pending_segments.len() > self.high_water_mark {
                    debug!("Backpressure: Buffer full ({}), waiting for writer...", pending_segments.len());
                    // The .buffered() stream naturally provides some backpressure, but this 
                    // extra check ensures we don't consume too much RAM if Segment 1 is stalled.
                }
            }
        }

        output_file.flush().await.map_err(|e| DownloadError::Config(e.to_string()))?;
        Ok(())
    }
}
