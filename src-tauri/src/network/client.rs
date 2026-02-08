/// Network client configuration and creation
///
/// This module provides HTTP client builders with pre-configured
/// settings for the download manager.

/// Creates an HTTP client for initial file metadata requests
///
/// # Returns
/// A configured `reqwest::Client` with user agent set
///
/// # Errors
/// Returns error if client builder fails
pub fn create_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
}

/// Creates an HTTP client for worker threads with aggressive timeouts
///
/// # Returns
/// A configured `reqwest::Client` with:
/// - User agent spoofing
/// - 5-second read timeout
/// - 5-second connect timeout
///
/// # Panics
/// Panics if client builder fails (should never happen with these settings)
pub fn create_worker_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .read_timeout(std::time::Duration::from_secs(5))
        .connect_timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("Failed to create worker HTTP client")
}
