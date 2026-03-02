use crate::core::error::DownloadError;
use crate::commands::DownloadCommandResult;
use std::sync::Arc;
use crate::commands;
use crate::core::state;

pub mod http;
pub mod hls;

/// The context required for a strategy to execute a download.
pub struct DownloadContext {
    pub app: tauri::AppHandle,
    pub download_id: String,
    pub metadata: state::DownloadMetadata,
    pub control: Arc<commands::DownloadControl>,
    pub manager: commands::DownloadManager,
    pub generation: u32,
}

/// A trait that defines a method for downloading a file.
/// This allows for different strategies (e.g., HTTP, HLS).
#[async_trait::async_trait]
pub trait DownloadStrategy: Send + Sync {
    async fn execute(&self, context: &DownloadContext) -> Result<DownloadCommandResult, DownloadError>;
}
