use super::{DownloadContext, DownloadStrategy};
use crate::commands::DownloadCommandResult;
use crate::core::error::DownloadError;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{info, error, warn};
use tauri::Emitter;
use std::sync::atomic::Ordering;

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
            "Starting FFmpeg download strategy"
        );

        // In a real bundled scenario, we'd use:
        // let ffmpeg_path = context.app.path().resolve_resource("ffmpeg.exe")...
        // For now, we assume it's in the PATH as per the initial plan phase.
        let mut cmd = Command::new("ffmpeg");
        
        cmd.arg("-y") // Overwrite output files
           .arg("-i")
           .arg(url)
           .arg("-c")
           .arg("copy") // Copy streams without re-encoding
           .arg(filepath)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            error!("Failed to spawn ffmpeg: {}", e);
            DownloadError::Config(format!("FFmpeg not found or failed to start: {}", e))
        })?;

        let stderr = child.stderr.take().unwrap();
        let mut reader = BufReader::new(stderr).lines();
        let app_handle = context.app.clone();
        let control = context.control.clone();

        // Monitor progress from stderr
        tokio::spawn(async move {
            while let Ok(Some(line)) = reader.next_line().await {
                // FFmpeg progress lines often contain "time=HH:MM:SS.MS"
                // For MVP, we'll just emit that it's "Working"
                // A more advanced parser would extract duration and calculate %
                if line.contains("time=") {
                    // Just a heartbeat for the UI to know we are alive
                    let _ = app_handle.emit("download-progress-pulse", line.trim().to_string());
                }
                
                if control.signal.load(Ordering::Relaxed) != 0 {
                    break;
                }
            }
        });

        // Monitor for cancellation/pause
        let control_for_kill = context.control.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let signal = control_for_kill.signal.load(Ordering::Relaxed);
                if signal != 0 {
                    // Try to kill ffmpeg gracefully or forcefully
                    // FFmpeg can be stopped by sending 'q' to stdin, but killing is easier for MVP
                    break;
                }
            }
        });

        let status = child.wait().await.map_err(|e| {
            DownloadError::TaskJoin(format!("FFmpeg process failed: {}", e))
        })?;

        if !status.success() {
            let signal = context.control.signal.load(Ordering::Relaxed);
            if signal != 0 {
                info!("FFmpeg stopped by user signal: {}", signal);
                return Ok(DownloadCommandResult {
                    id: context.download_id.clone(),
                    status: if signal == 1 { "paused".to_string() } else { "cancelled".to_string() },
                });
            }
            return Err(DownloadError::Network("FFmpeg exited with error".to_string()));
        }

        info!(download_id = %context.download_id, "FFmpeg download completed");

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
