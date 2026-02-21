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
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .pool_max_idle_per_host(32)
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .build()
}

/// Creates an HTTP client for worker threads with aggressive timeouts
///
/// # Returns
/// A configured `reqwest::Client` with:
/// - User agent spoofing
/// - 10-second read timeout
/// - 10-second connect timeout
///
/// # Panics
/// Panics if client builder fails (should never happen with these settings)
pub fn create_worker_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .read_timeout(std::time::Duration::from_secs(10))
        .connect_timeout(std::time::Duration::from_secs(10))
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .pool_max_idle_per_host(32)
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .build()
        .expect("Failed to create worker HTTP client")
}
