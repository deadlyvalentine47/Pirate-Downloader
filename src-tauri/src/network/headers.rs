/// HTTP header parsing utilities
use reqwest::header::CONTENT_DISPOSITION;

/// Extracts filename from HTTP response headers or URL
///
/// Tries in order:
/// 1. Content-Disposition header
/// 2. URL path segments
/// 3. Falls back to "download.dat"
///
/// # Arguments
/// * `response` - The HTTP response to extract filename from
/// * `url` - The original URL (used as fallback)
///
/// # Returns
/// The extracted filename, sanitized and ready to use
pub fn extract_filename(response: &reqwest::Response, url: &str) -> String {
    let mut filename = "download.dat".to_string();

    // Try Content-Disposition header first
    if let Some(disp) = response.headers().get(CONTENT_DISPOSITION) {
        if let Ok(disp_str) = disp.to_str() {
            if let Some(name_part) = disp_str.split("filename=").nth(1) {
                filename = name_part
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                return filename;
            }
        }
    }

    // Fallback to URL path
    if let Ok(parsed_url) = url::Url::parse(url) {
        if let Some(segments) = parsed_url.path_segments() {
            if let Some(last) = segments.last() {
                if !last.is_empty() {
                    filename = last.to_string();
                }
            }
        }
    }

    sanitize_filename::sanitize(filename)
}
