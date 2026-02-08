/// Tauri command modules
///
/// This module organizes all Tauri commands into logical groups
pub mod download_control;

// Re-export DownloadManager and DownloadControl for use in lib.rs
pub use download_control::{DownloadControl, DownloadManager};
