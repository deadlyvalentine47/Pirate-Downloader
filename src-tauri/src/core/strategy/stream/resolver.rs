use crate::core::error::DownloadError;
use reqwest::Client;
use url::Url;
use tracing::{info, debug};

#[async_trait::async_trait]
pub trait StreamResolver: Send + Sync {
    async fn resolve(&self, url: &str, client: &Client, headers: &reqwest::header::HeaderMap) -> Result<Vec<String>, DownloadError>;
}

pub struct HlsResolver;

#[async_trait::async_trait]
impl StreamResolver for HlsResolver {
    async fn resolve(&self, url: &str, client: &Client, headers: &reqwest::header::HeaderMap) -> Result<Vec<String>, DownloadError> {
        debug!("HLS Resolver: Fetching manifest from {}", url);
        
        let response = client.get(url)
            .headers(headers.clone())
            .send()
            .await
            .map_err(|e| DownloadError::Network(format!("Failed to fetch HLS manifest: {}", e)))?;

        if !response.status().is_success() {
            return Err(DownloadError::Network(format!("Server returned error: {}", response.status())));
        }

        let text = response.text().await
            .map_err(|e| DownloadError::Network(format!("Failed to read manifest body: {}", e)))?;

        let base_url = Url::parse(url).map_err(|e| DownloadError::Config(format!("Invalid base URL: {}", e)))?;
        let mut segment_urls = Vec::new();

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                // Check if it's a pointer to a sub-manifest (Quality Selection)
                if trimmed.contains(".m3u8") && !trimmed.starts_with("#EXT") {
                    let sub_url = base_url.join(trimmed).map_err(|e| DownloadError::Config(e.to_string()))?;
                    info!("HLS Resolver: Detected sub-manifest, recursing to: {}", sub_url);
                    // Recurse to get the actual segments from the sub-manifest
                    return Box::pin(self.resolve(sub_url.as_str(), client, headers)).await;
                }
                continue;
            }

            // Resolve relative URLs
            let full_url = base_url.join(trimmed)
                .map_err(|e| DownloadError::Config(format!("Failed to resolve segment URL: {}", e)))?;
            segment_urls.push(full_url.to_string());
        }

        if segment_urls.is_empty() {
            return Err(DownloadError::Config("No segments found in HLS manifest".to_string()));
        }

        Ok(segment_urls)
    }
}
