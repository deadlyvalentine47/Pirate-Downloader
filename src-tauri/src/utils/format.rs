/// Utility for detecting media formats and streaming protocols.

pub fn is_streaming_protocol(url: &str) -> bool {
    let url_lc = url.to_lowercase();
    
    // Streaming manifest files (using contains to handle query params)
    if url_lc.contains(".m3u8") || 
       url_lc.contains(".mpd") || 
       url_lc.contains(".ism") || 
       url_lc.contains(".f4m") {
        return true;
    }

    // Streaming protocols
    if url_lc.contains("rtmp://") || 
       url_lc.contains("rtmps://") ||
       url_lc.contains("rtsp://") || 
       url_lc.contains("rtsps://") ||
       url_lc.contains("mms://") || 
       url_lc.contains("mmsh://") ||
       url_lc.contains("srt://") {
        return true;
    }

    false
}

pub fn get_output_container(url: &str) -> &str {
    let url_lc = url.to_lowercase();
    
    if url_lc.contains(".m3u8") || url_lc.starts_with("srt://") {
        "ts"
    } else if url_lc.contains(".mpd") || url_lc.contains(".ism") {
        "mp4"
    } else if url_lc.starts_with("rtmp") || url_lc.contains(".f4m") {
        "flv"
    } else {
        "mkv" // Universal fallback
    }
}
