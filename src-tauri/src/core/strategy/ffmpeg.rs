use super::{DownloadContext, DownloadStrategy};
use crate::commands::DownloadCommandResult;
use crate::core::error::DownloadError;
use tauri_plugin_shell::ShellExt;
use tracing::{info, error};
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
            "Starting FFmpeg sidecar download strategy"
        );

        let sidecar = context.app.shell().sidecar("ffmpeg").map_err(|e| {
            error!("Failed to create ffmpeg sidecar: {}", e);
            DownloadError::Config(format!("FFmpeg sidecar error: {}", e))
        })?;

        // Construct headers for FFmpeg: "Key: Value\r\nKey2: Value2\r\n"
        let mut header_str = String::new();
        for (k, v) in &context.metadata.headers {
            header_str.push_str(&format!("{}: {}\r\n", k, v));
        }
        if let Some(ref_url) = &context.metadata.referrer {
            header_str.push_str(&format!("Referer: {}\r\n", ref_url));
        }

        let mut args = vec!["-y"];
        
        if !header_str.is_empty() {
            args.push("-headers");
            args.push(&header_str);
        }

        args.push("-i");
        args.push(url);
        args.push("-c");
        args.push("copy");
        args.push(filepath);

        let (mut rx, _child) = sidecar
            .args(args)
            .spawn()
            .map_err(|e| {
                error!("Failed to spawn ffmpeg sidecar: {}", e);
                DownloadError::Config(format!("FFmpeg failed to start: {}", e))
            })?;

        let app_handle = context.app.clone();
        let control = context.control.clone();

        // Monitor progress from sidecar events
        while let Some(event) = rx.recv().await {
            match event {
                tauri_plugin_shell::process::CommandEvent::Stderr(line) => {
                    let line_str = String::from_utf8_lossy(&line);
                    if line_str.contains("time=") {
                        let _ = app_handle.emit("download-progress-pulse", line_str.trim().to_string());
                    }
                }
                tauri_plugin_shell::process::CommandEvent::Terminated(payload) => {
                    if payload.code == Some(0) {
                        info!(download_id = %context.download_id, "FFmpeg sidecar completed successfully");
                        break;
                    } else {
                        let signal = control.signal.load(Ordering::Relaxed);
                        if signal != 0 {
                            info!("FFmpeg sidecar stopped by user signal");
                            return Ok(DownloadCommandResult {
                                id: context.download_id.clone(),
                                status: if signal == 1 { "paused".to_string() } else { "cancelled".to_string() },
                            });
                        }
                        return Err(DownloadError::Network(format!("FFmpeg sidecar exited with code {:?}", payload.code)));
                    }
                }
                _ => {}
            }
            
            if control.signal.load(Ordering::Relaxed) != 0 {
                // How to kill sidecar? _child.kill() if we kept it.
                // In v2, the child is managed by the plugin.
                break;
            }
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
