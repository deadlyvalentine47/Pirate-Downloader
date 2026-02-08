/// Logging configuration and initialization
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize the logging system
///
/// In development mode (debug builds), logs are written to both console and file.
/// In production mode (release builds), logs are written to file only.
///
/// Log levels:
/// - Development: DEBUG and above
/// - Production: INFO and above
///
/// Log file location: %APPDATA%/PirateDownloader/logs/
/// Log file naming: pirate-downloader-YYYY-MM-DD.log
/// Rotation: Daily or when file reaches 10MB
pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    // Determine log directory
    let log_dir = get_log_directory()?;

    // Create log directory if it doesn't exist
    std::fs::create_dir_all(&log_dir)?;

    // Set up file appender with daily rotation
    let file_appender = tracing_appender::rolling::daily(&log_dir, "pirate-downloader.log");
    let (non_blocking_file, _guard) = tracing_appender::non_blocking(file_appender);

    // Determine log level based on build type
    #[cfg(debug_assertions)]
    let log_level = "debug";

    #[cfg(not(debug_assertions))]
    let log_level = "info";

    // Create environment filter
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    // Build the subscriber
    #[cfg(debug_assertions)]
    {
        // Development: Console + File
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .with_writer(std::io::stdout)
                    .with_target(false)
                    .with_thread_ids(false)
                    .with_file(false)
                    .with_line_number(false),
            )
            .with(fmt::layer().with_writer(non_blocking_file).with_ansi(false))
            .init();
    }

    #[cfg(not(debug_assertions))]
    {
        // Production: File only
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().with_writer(non_blocking_file).with_ansi(false))
            .init();
    }

    // Log initialization message
    tracing::info!("Logger initialized - log directory: {}", log_dir.display());

    Ok(())
}

/// Get the log directory path
///
/// Returns: %APPDATA%/PirateDownloader/logs/ on Windows
fn get_log_directory() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let app_data =
        std::env::var("APPDATA").map_err(|_| "APPDATA environment variable not found")?;

    let mut log_dir = std::path::PathBuf::from(app_data);
    log_dir.push("PirateDownloader");
    log_dir.push("logs");

    Ok(log_dir)
}

/// Get the current log file path (for debugging/testing)
#[allow(dead_code)]
pub fn get_current_log_file() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let log_dir = get_log_directory()?;
    let today = chrono::Local::now().format("%Y-%m-%d");
    let log_file = log_dir.join(format!("pirate-downloader-{}.log", today));
    Ok(log_file)
}
