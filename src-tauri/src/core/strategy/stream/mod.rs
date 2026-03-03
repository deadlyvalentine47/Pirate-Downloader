pub mod downloader;
pub mod processor;
pub mod resolver;
pub mod youtube;

use super::{DownloadContext, DownloadStrategy};
use crate::commands::DownloadCommandResult;
use crate::core::error::DownloadError;
use std::sync::Arc;
use reqwest::Client;
use downloader::ParallelDownloader;
use processor::StreamProcessor;
use resolver::{StreamResolver, HlsResolver};
use youtube::YoutubeResolver;
use tokio::fs::OpenOptions;
use tracing::{info, debug};

pub struct StreamingConfig {
    pub enable_parallel_segments: bool,
    pub enable_header_stripping: bool,
    pub enable_platform_resolvers: bool,
    pub max_parallel_connections: usize,
    pub buffer_high_water_mark: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            enable_parallel_segments: true,
            enable_header_stripping: true,
            enable_platform_resolvers: true,
            max_parallel_connections: 16,
            buffer_high_water_mark: 32,
        }
    }
}

pub struct UniversalStreamingStrategy {
    config: StreamingConfig,
    client: Arc<Client>,
    downloader: Arc<ParallelDownloader>,
    processor: Arc<StreamProcessor>,
    hls_resolver: Arc<HlsResolver>,
    youtube_resolver: Arc<YoutubeResolver>,
}

impl UniversalStreamingStrategy {
    pub fn new(config: Option<StreamingConfig>) -> Self {
        let config = config.unwrap_or_default();
        let client = Arc::new(crate::network::client::create_client().unwrap_or_else(|_| Client::new()));
        let processor = Arc::new(StreamProcessor::new(config.enable_header_stripping));
        let hls_resolver = Arc::new(HlsResolver);
        let youtube_resolver = Arc::new(YoutubeResolver);
        let downloader = Arc::new(ParallelDownloader::new(
            client.clone(),
            processor.clone(),
            config.max_parallel_connections,
            config.buffer_high_water_mark,
        ));

        Self {
            config,
            client,
            downloader,
            processor,
            hls_resolver,
            youtube_resolver,
        }
    }
}

#[async_trait::async_trait]
impl DownloadStrategy for UniversalStreamingStrategy {
    async fn execute(
        &self,
        context: &DownloadContext,
    ) -> Result<DownloadCommandResult, DownloadError> {
        let url = &context.metadata.url;
        let filepath = &context.metadata.filepath;

        info!(download_id = %context.download_id, "Universal Engine: Starting download for {}", url);

        // 1. Prepare Headers
        let mut header_map = reqwest::header::HeaderMap::new();
        for (k, v) in &context.metadata.headers {
            if let (Ok(name), Ok(val)) = (
                reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                reqwest::header::HeaderValue::from_str(v),
            ) {
                header_map.insert(name, val);
            }
        }

        // Add standard User-Agent and Referer if present
        header_map.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"),
        );
        if let Some(ref_url) = &context.metadata.referrer {
            if let Ok(val) = reqwest::header::HeaderValue::from_str(ref_url) {
                header_map.insert(reqwest::header::REFERER, val);
            }
        }

        // 2. Routing Logic
        let segment_urls = if url.contains("youtube.com") || url.contains("youtu.be") {
            if !self.config.enable_platform_resolvers {
                return Err(DownloadError::Config("Platform resolvers are currently disabled".to_string()));
            }
            self.youtube_resolver.resolve(url, &self.client, &header_map).await?
        } else {
            // Default to HLS
            self.hls_resolver.resolve(url, &self.client, &header_map).await?
        };

        debug!(download_id = %context.download_id, "Resolved {} segments", segment_urls.len());

        // 3. Prepare File
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(filepath)
            .await
            .map_err(|e| DownloadError::Config(format!("Failed to create output file: {}", e)))?;

        // 4. Download
        self.downloader.download_segments(
            segment_urls,
            header_map,
            file,
            context.control.signal.clone()
        ).await?;

        info!(download_id = %context.download_id, "Universal Engine: Download complete");
        
        Ok(DownloadCommandResult {
            id: context.download_id.clone(),
            status: "completed".to_string(),
        })
    }
}
