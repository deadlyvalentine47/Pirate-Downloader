pub mod downloader;

use super::{DownloadContext, DownloadStrategy};
use crate::commands::DownloadCommandResult;
use crate::core::error::DownloadError;
use std::sync::Arc;
use reqwest::Client;
use downloader::ParallelDownloader;

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
}

impl UniversalStreamingStrategy {
    pub fn new(config: Option<StreamingConfig>) -> Self {
        let config = config.unwrap_or_default();
        let client = Arc::new(crate::network::client::create_client().unwrap_or_else(|_| Client::new()));
        let downloader = Arc::new(ParallelDownloader::new(
            client.clone(),
            config.max_parallel_connections,
            config.buffer_high_water_mark,
        ));

        Self {
            config,
            client,
            downloader,
        }
    }
}

#[async_trait::async_trait]
impl DownloadStrategy for UniversalStreamingStrategy {
    async fn execute(
        &self,
        _context: &DownloadContext,
    ) -> Result<DownloadCommandResult, DownloadError> {
        // Placeholder for phase 2/3: Orchestration between Resolver, Downloader, and Processor
        Err(DownloadError::Config("Universal Streaming Strategy not yet fully implemented".to_string()))
    }
}
