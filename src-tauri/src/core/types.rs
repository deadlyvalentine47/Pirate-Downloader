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
