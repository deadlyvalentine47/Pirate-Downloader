use serde::{Deserialize, Serialize};
/// Core download engine types and constants
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::sync::Mutex;

// Type aliases for future use in MVP2 features
#[allow(dead_code)]
/// Queue of chunk indices to be downloaded
pub type ChunkQueue = Arc<Mutex<VecDeque<u64>>>;

#[allow(dead_code)]
/// Tracks retry attempts per chunk ID
pub type RetryTracker = Arc<Mutex<HashMap<u64, u32>>>;

#[allow(dead_code)]
/// Atomic counter for completed chunks
pub type CompletionCounter = Arc<AtomicU64>;

/// Atomic counter for downloaded bytes
#[allow(dead_code)]
pub type ByteCounter = Arc<AtomicU64>;

#[allow(dead_code)]
/// Speed statistics: (last_bytes, peak_speed, min_speed, last_update_time)
pub type SpeedStats = Arc<Mutex<(u64, f64, f64, std::time::Instant)>>;

/// Default number of download threads if not specified
pub const DEFAULT_THREADS: u64 = 8;

/// Maximum retry attempts per chunk before giving up
pub const CHUNK_RETRY_LIMIT: u64 = 5;

/// Minimum speed threshold in KB/s before killing slow chunks
pub const SPEED_ENFORCEMENT_THRESHOLD: f64 = 300.0;

/// Time to wait before enforcing speed threshold (seconds)
pub const SPEED_ENFORCEMENT_DELAY: f64 = 3.0;

/// Retry count threshold - disable speed enforcement after this many retries
pub const ADAPTIVE_RETRY_THRESHOLD: u32 = 3;

/// Download status representing the current state of a download
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum DownloadStatus {
    /// Download is queued but not started
    Pending,
    /// Download is actively running
    Active,
    /// Download is temporarily paused (can resume)
    Paused,
    /// Download is stopped (can resume later)
    Stopped,
    /// Download completed successfully
    Completed,
    /// Download failed with errors
    Failed,
    /// Download was cancelled and cleaned up
    Cancelled,
}

impl std::fmt::Display for DownloadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadStatus::Pending => write!(f, "pending"),
            DownloadStatus::Active => write!(f, "active"),
            DownloadStatus::Paused => write!(f, "paused"),
            DownloadStatus::Stopped => write!(f, "stopped"),
            DownloadStatus::Completed => write!(f, "completed"),
            DownloadStatus::Failed => write!(f, "failed"),
            DownloadStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Complete download state for persistence and resume
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct DownloadState {
    /// Unique identifier for this download
    pub id: String,
    /// Download URL
    pub url: String,
    /// Target file path (without .part extension)
    pub filepath: String,
    /// Total file size in bytes
    pub total_size: u64,
    /// Bytes downloaded so far
    pub downloaded_bytes: u64,
    /// Current download status
    pub status: DownloadStatus,
    /// Number of download threads
    pub thread_count: u64,
    /// Chunk size in bytes
    pub chunk_size: u64,
    /// List of completed chunk indices
    pub completed_chunks: Vec<u64>,
    /// Timestamp when download was created
    pub created_at: String,
    /// Timestamp of last state change
    pub updated_at: String,
    /// Timestamp when download was paused (if applicable)
    pub paused_at: Option<String>,
    /// Timestamp when download was stopped (if applicable)
    pub stopped_at: Option<String>,
    /// Timestamp when download completed (if applicable)
    pub completed_at: Option<String>,
    /// Average download speed in bytes/sec
    pub avg_speed: Option<f64>,
    /// Error message if failed
    pub error_message: Option<String>,
}

#[allow(dead_code)]
impl DownloadState {
    /// Create a new download state
    pub fn new(
        id: String,
        url: String,
        filepath: String,
        total_size: u64,
        thread_count: u64,
        chunk_size: u64,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id,
            url,
            filepath,
            total_size,
            downloaded_bytes: 0,
            status: DownloadStatus::Pending,
            thread_count,
            chunk_size,
            completed_chunks: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
            paused_at: None,
            stopped_at: None,
            completed_at: None,
            avg_speed: None,
            error_message: None,
        }
    }

    /// Calculate completion percentage
    pub fn completion_percentage(&self) -> f64 {
        if self.total_size == 0 {
            0.0
        } else {
            (self.downloaded_bytes as f64 / self.total_size as f64) * 100.0
        }
    }

    /// Get the path to the partial file (.part)
    pub fn part_filepath(&self) -> String {
        format!("{}.part", self.filepath)
    }

    /// Get the path to the state file (.part.state)
    pub fn state_filepath(&self) -> String {
        format!("{}.part.state", self.filepath)
    }
}
