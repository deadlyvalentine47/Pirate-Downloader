use chrono::{DateTime, Utc};
/// Download state management
///
/// This module defines the state machine for downloads and provides
/// serialization/deserialization for state persistence.
use serde::{Deserialize, Serialize};

/// Download state enum - represents all possible states a download can be in
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadState {
    /// Download is queued but not started
    Pending,
    /// Download is actively running
    Active,
    /// Download is paused by user (can resume)
    Paused,
    /// Download is stopped gracefully (can resume later)
    Stopped,
    /// Download completed successfully
    Completed,
    /// Download failed due to error
    Failed,
    /// Download was cancelled by user (cleanup performed)
    Cancelled,
}

impl DownloadState {
    /// Check if download can be resumed from this state
    pub fn can_resume(&self) -> bool {
        matches!(self, DownloadState::Paused | DownloadState::Stopped)
    }

    /// Check if download is in a terminal state (no further action possible)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            DownloadState::Completed | DownloadState::Failed | DownloadState::Cancelled
        )
    }

    /// Check if download is currently active
    pub fn is_active(&self) -> bool {
        matches!(self, DownloadState::Active)
    }
}

/// Download metadata - all information needed to resume a download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadMetadata {
    /// Original download URL
    pub url: String,

    /// Full path to the download file (including .part extension)
    pub filepath: String,

    /// Total file size in bytes
    pub total_size: u64,

    /// Bytes downloaded so far
    pub downloaded_bytes: u64,

    /// Current download state
    pub state: DownloadState,

    /// Number of threads used for download
    pub thread_count: u32,

    /// List of completed chunk IDs
    pub completed_chunks: Vec<u64>,

    /// List of incomplete chunk IDs
    pub incomplete_chunks: Vec<u64>,

    /// When the download was created
    pub created_at: DateTime<Utc>,

    /// When the download was last paused (if applicable)
    pub paused_at: Option<DateTime<Utc>>,

    /// When the download was last resumed (if applicable)
    pub resumed_at: Option<DateTime<Utc>>,

    /// When the download was stopped (if applicable)
    pub stopped_at: Option<DateTime<Utc>>,

    /// When the download completed (if applicable)
    pub completed_at: Option<DateTime<Utc>>,

    /// Error message if download failed
    pub error_message: Option<String>,
}

impl DownloadMetadata {
    /// Create new metadata for a fresh download
    pub fn new(url: String, filepath: String, total_size: u64, thread_count: u32) -> Self {
        Self {
            url,
            filepath,
            total_size,
            downloaded_bytes: 0,
            state: DownloadState::Pending,
            thread_count,
            completed_chunks: Vec::new(),
            incomplete_chunks: Vec::new(),
            created_at: Utc::now(),
            paused_at: None,
            resumed_at: None,
            stopped_at: None,
            completed_at: None,
            error_message: None,
        }
    }

    /// Calculate download progress percentage
    pub fn progress_percentage(&self) -> f64 {
        if self.total_size == 0 {
            0.0
        } else {
            (self.downloaded_bytes as f64 / self.total_size as f64) * 100.0
        }
    }

    /// Mark download as paused
    pub fn pause(&mut self) {
        self.state = DownloadState::Paused;
        self.paused_at = Some(Utc::now());
    }

    /// Mark download as resumed
    pub fn resume(&mut self) {
        self.state = DownloadState::Active;
        self.resumed_at = Some(Utc::now());
    }

    /// Mark download as stopped
    pub fn stop(&mut self) {
        self.state = DownloadState::Stopped;
        self.stopped_at = Some(Utc::now());
    }

    /// Mark download as completed
    pub fn complete(&mut self) {
        self.state = DownloadState::Completed;
        self.completed_at = Some(Utc::now());
    }

    /// Mark download as failed
    pub fn fail(&mut self, error: String) {
        self.state = DownloadState::Failed;
        self.error_message = Some(error);
    }

    /// Mark download as cancelled
    pub fn cancel(&mut self) {
        self.state = DownloadState::Cancelled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transitions() {
        assert!(DownloadState::Paused.can_resume());
        assert!(DownloadState::Stopped.can_resume());
        assert!(!DownloadState::Active.can_resume());
        assert!(!DownloadState::Completed.can_resume());
    }

    #[test]
    fn test_terminal_states() {
        assert!(DownloadState::Completed.is_terminal());
        assert!(DownloadState::Failed.is_terminal());
        assert!(DownloadState::Cancelled.is_terminal());
        assert!(!DownloadState::Active.is_terminal());
    }

    #[test]
    fn test_metadata_creation() {
        let meta = DownloadMetadata::new(
            "https://example.com/file.zip".to_string(),
            "/tmp/file.zip.part".to_string(),
            1024,
            16,
        );

        assert_eq!(meta.state, DownloadState::Pending);
        assert_eq!(meta.downloaded_bytes, 0);
        assert_eq!(meta.progress_percentage(), 0.0);
    }

    #[test]
    fn test_progress_calculation() {
        let mut meta = DownloadMetadata::new(
            "https://example.com/file.zip".to_string(),
            "/tmp/file.zip.part".to_string(),
            1000,
            16,
        );

        meta.downloaded_bytes = 500;
        assert_eq!(meta.progress_percentage(), 50.0);

        meta.downloaded_bytes = 1000;
        assert_eq!(meta.progress_percentage(), 100.0);
    }
}
