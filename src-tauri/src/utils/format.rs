/// Utility for detecting media formats and streaming protocols.

pub fn requires_ffmpeg(url: &str) -> bool {
    let url_lc = url.to_lowercase();
    
    // Streaming manifest files
    if url_lc.contains(".m3u8") || 
       url_lc.contains(".mpd") || 
       url_lc.contains(".ism") || 
       url_lc.contains(".f4m") {
        return true;
    }

    // Streaming protocols
    if url_lc.starts_with("rtmp://") || 
       url_lc.starts_with("rtmps://") ||
       url_lc.starts_with("rtsp://") || 
       url_lc.starts_with("rtsps://") ||
       url_lc.starts_with("mms://") || 
       url_lc.starts_with("mmsh://") ||
       url_lc.starts_with("srt://") {
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
        "mkv" // Universal fallback for ffmpeg
    }
}
