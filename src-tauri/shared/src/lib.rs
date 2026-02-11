use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum IpcMessage {
    /// Request to start a download
    DownloadRequest(DownloadRequest),
    /// Simple ping to check if server is alive
    Ping,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub filename: Option<String>,
    pub headers: std::collections::HashMap<String, String>,
    pub cookies: Option<String>,
    pub referrer: Option<String>,
}

/// The name of the pipe/socket to connect to
#[cfg(windows)]
pub const IPC_NAME: &str = r"\\.\pipe\pirate-downloader-ipc";

#[cfg(not(windows))]
pub const IPC_NAME: &str = "/tmp/pirate-downloader-ipc.sock";
