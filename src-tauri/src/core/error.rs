/// Custom error types for the download application

/// Main error type for download operations
#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum DownloadError {
    /// Network-related errors (HTTP requests, connections)
    #[error("Network error: {0}")]
    Network(String),

    /// File system errors (I/O, permissions, disk space)
    #[error("File system error: {0}")]
    FileSystem(String),

    /// Download integrity errors (incomplete, verification failed)
    #[error("Integrity check failed: {message}")]
    Integrity { message: String },

    /// Parsing errors (headers, URLs, configuration)
    #[error("Parse error: {0}")]
    Parse(String),

    /// Configuration errors (invalid settings, missing values)
    #[error("Configuration error: {0}")]
    Config(String),

    /// Task join errors (thread/async task failures)
    #[error("Task join error: {0}")]
    TaskJoin(String),
}

/// Helper trait to add context to errors
pub trait ErrorContext<T> {
    fn context(self, msg: &str) -> Result<T, DownloadError>;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: Into<DownloadError>,
{
    fn context(self, msg: &str) -> Result<T, DownloadError> {
        self.map_err(|e| {
            let base_error = e.into();
            match base_error {
                DownloadError::Network(e) => DownloadError::Network(e),
                DownloadError::FileSystem(e) => DownloadError::FileSystem(e),
                _ => DownloadError::Config(format!("{}: {:?}", msg, base_error)),
            }
        })
    }
}

/// Convert tokio::task::JoinError to DownloadError
impl From<tokio::task::JoinError> for DownloadError {
    fn from(err: tokio::task::JoinError) -> Self {
        DownloadError::TaskJoin(err.to_string())
    }
}

/// Convert reqwest::Error to DownloadError
impl From<reqwest::Error> for DownloadError {
    fn from(err: reqwest::Error) -> Self {
        DownloadError::Network(err.to_string())
    }
}

/// Convert std::io::Error to DownloadError
impl From<std::io::Error> for DownloadError {
    fn from(err: std::io::Error) -> Self {
        DownloadError::FileSystem(err.to_string())
    }
}

/// Convert String errors to DownloadError (for compatibility with existing code)
impl From<String> for DownloadError {
    fn from(err: String) -> Self {
        DownloadError::Config(err)
    }
}

/// Convert &str errors to DownloadError
impl From<&str> for DownloadError {
    fn from(err: &str) -> Self {
        DownloadError::Config(err.to_string())
    }
}
