use crate::commands::DownloadCommandResult;
use crate::core::error::DownloadError;
use crate::core::state;
use crate::core::strategy::{DownloadContext, DownloadStrategy};
use std::sync::Arc;
use crate::commands;

pub struct DownloadEngine;

impl DownloadEngine {
    /// Starts a download using a provided strategy.
    pub async fn start(
        app: tauri::AppHandle,
        download_id: String,
        metadata: state::DownloadMetadata,
        control: Arc<commands::DownloadControl>,
        manager: commands::DownloadManager,
        generation: u32,
        strategy: Box<dyn DownloadStrategy>,
    ) -> Result<DownloadCommandResult, DownloadError> {
        // 1. Construct the context
        let context = DownloadContext {
            app,
            download_id,
            metadata,
            control,
            manager,
            generation,
        };

        // 2. Execute the strategy
        strategy.execute(&context).await
    }
}
