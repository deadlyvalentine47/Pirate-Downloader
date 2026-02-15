Universal Streaming Format Support
Problem Analysis
Current Behavior
Our download engine only handles direct HTTP downloads (regular files). It fails for:

Adaptive Streaming Formats (segmented):

.m3u8 (HLS) - Apple's HTTP Live Streaming
.mpd (MPEG-DASH) - Dynamic Adaptive Streaming
.ism (Smooth Streaming) - Microsoft's format
.f4m (HDS) - Adobe HTTP Dynamic Streaming
Real-Time Streaming Protocols:

rtmp:// - Real-Time Messaging Protocol
rtsp:// - Real-Time Streaming Protocol
mms:// - Microsoft Media Server
Other Protocols:

srt:// - Secure Reliable Transport
Live streams (YouTube, Twitch, etc.)
What IDM/yt-dlp Do
They use ffmpeg as the universal solution:

Detects ANY streaming format/protocol
Downloads and processes automatically
Handles encryption, authentication, adaptive bitrate
Outputs standard video file (.mp4, 
.ts
, .mkv)
Our Gap
We only download what reqwest can fetch directly via HTTP GET. Everything else fails.

Solution: Universal ffmpeg Integration ⭐
Why ffmpeg is the ONLY practical solution:

✅ Supports 100+ streaming protocols out of the box
✅ Handles ALL major formats: HLS, DASH, RTMP, RTSP, SRT, etc.
✅ Battle-tested by YouTube, Twitch, Netflix downloaders
✅ Automatic format detection - no manual parsing needed
✅ One command: ffmpeg -i {url} -c copy {output}
✅ No complex codec, encryption, or DRM logic needed
Alternative (building it ourselves):

❌ Would need 10+ different parsers (HLS, DASH, RTMP client, etc.)
❌ 5,000-10,000 lines of complex code
❌ Months of development and testing
❌ Still wouldn't match ffmpeg's capabilities
Format Detection Strategy
Smart URL Detection
rust
fn requires_ffmpeg(url: &str) -> bool {
    // Streaming manifest files
    url.contains(".m3u8") || url.contains(".mpd") || 
    url.contains(".ism") || url.contains(".f4m") ||
    
    // Streaming protocols
    url.starts_with("rtmp://") || url.starts_with("rtmps://") ||
    url.starts_with("rtsp://") || url.starts_with("rtsps://") ||
    url.starts_with("mms://") || url.starts_with("mmsh://") ||
    url.starts_with("srt://")
}
Content-Type Detection (Fallback)
For URLs without clear indicators, check response headers:

application/vnd.apple.mpegurl → HLS
application/dash+xml → DASH
Recommended Implementation
Phase 1: ffmpeg Integration (THIS SPRINT) ⭐
Architecture:

┌─────────────────┐
│ Extension/User  │
└────────┬────────┘
         │ Download Request
         ▼
┌─────────────────────────┐
│   Download Router       │
│   (ipc.rs)              │
└────────┬────────────────┘
         │
    ┌────┴────┐
    │ Detect  │
    └────┬────┘
         │
    ┌────┴──────────────────┐
    │                       │
    ▼                       ▼
┌─────────────┐    ┌──────────────────┐
│ Direct HTTP │    │ Streaming Format │
│ (Our Engine)│    │ (ffmpeg)         │
└─────────────┘    └──────────────────┘
1. Detection Layer (
src-tauri/src/utils/format.rs
):

rust
pub fn requires_ffmpeg(url: &str) -> bool {
    // Comprehensive detection logic
}
pub fn get_output_container(url: &str) -> &str {
    if url.contains(".m3u8") { "ts" }
    else if url.contains(".mpd") { "mp4" }
    else if url.starts_with("rtmp") { "flv" }
    else { "mkv" } // Universal fallback
}
2. ffmpeg Module (
src-tauri/src/core/ffmpeg.rs
):

rust
pub struct FfmpegDownloader;
impl FfmpegDownloader {
    pub async fn download(
        url: &str,
        output: &Path,
        referrer: Option<&str>,
        progress_tx: UnboundedSender<DownloadProgress>
    ) -> Result<(), DownloadError> {
        // 1. Check ffmpeg availability
        // 2. Build command with headers
        // 3. Execute and parse progress
        // 4. Report to UI
    }
}
3. Download Router (in 
ipc.rs
):

rust
let output_ext = if requires_ffmpeg(&req.url) {
    format::get_output_container(&req.url)
} else {
    // Use suggested filename extension
};
if requires_ffmpeg(&req.url) {
    ffmpeg::FfmpegDownloader::download(
        &req.url,
        &output_path,
        req.referrer.as_deref(),
        progress_tx
    ).await?
} else {
    DownloadEngine::start(...).await?
}
4. User Experience:

✅ ffmpeg available: Seamless download of ANY format
❌ ffmpeg missing: Modal shows: "Install ffmpeg to download streaming videos"
Link to: https://ffmpeg.org/download.html
Or bundled installer option (future)
Phase 2: Progress Parsing (Same Sprint)
Parse ffmpeg's stderr for progress updates:

frame= 1234 fps=30 time=00:01:23.45 bitrate=1234.5kbits/s speed=2.0x
Extract:

time → Elapsed download time
speed → Download speed multiplier
Estimate total time if possible
Phase 3: ffmpeg Bundling (Future)
Include ffmpeg binary in installer
Auto-download on first streaming URL
No user intervention needed
Verification Plan
Test Cases
HLS (.m3u8): The Lord of the Rings URL (current failing case)
Direct MP4: Regular video file (should still work)
DASH (.mpd): YouTube/Netflix style streaming
RTMP stream: Live stream test
RTSP: IP camera stream
No ffmpeg: Should show helpful error with download link
Success Criteria
✅ All streaming formats download successfully
✅ Output files are valid, playable videos
✅ Progress shows in UI (even if approximate)
✅ Direct HTTP downloads still work (existing engine)
✅ Referrer headers passed correctly
✅ Graceful error when ffmpeg missing
Format Coverage Table
Format	Extension	Protocol	Example Use Case	ffmpeg Support
HLS	.m3u8	HTTP	Apple, most streaming	✅ Native
DASH	.mpd	HTTP	YouTube, Netflix	✅ Native
Smooth	.ism	HTTP	Legacy Microsoft	✅ Native
HDS	.f4m	HTTP	Legacy Adobe	✅ Native
RTMP	-	rtmp://	Live broadcasts	✅ Native
RTSP	-	rtsp://	IP cameras	✅ Native
SRT	-	srt://	Low-latency	✅ Native
MMS	-	mms://	Legacy Windows	✅ Native
Direct	.mp4, .mkv, etc.	HTTP	Regular files	⚡ Our engine
Universal Streaming Support (Phase 5) [COMPLETED]
Goal
Enable downloading of HLS (.m3u8), DASH (.mpd), and other streaming formats using FFmpeg.

Component Updates
Core
 Create 
src-tauri/src/core/ffmpeg.rs
 wrapper
 Add mechanism to check for FFmpeg availability
 Implement 
FfmpegDownloader
 struct
Utils
 Create 
src-tauri/src/utils/format.rs
 for detection
 Implement 
requires_ffmpeg(url)
 and 
get_output_container(url)
IPC & Routing
 Update 
src-tauri/src/ipc.rs
 to detect streaming URLs in 
handle_download_request
 Suggest correct extension (e.g., .ts for HLS) in confirmation dialog
 Pass 
referrer
 from IPC to Frontend to Backend
Lib & Manager
 Update 
download_file
 and 
start_download
 to accept 
referrer
 Route streaming URLs to 
FfmpegDownloader
 instead of 
DownloadEngine
 Update 
DownloadManager
 to support external status updates
Verification
 Verify compilation
 User Manual Test: Download HLS stream (e.g. from the provided log URL)
3. Implement ffmpeg Downloader
 FfmpegDownloader::download() function
 Build command: ffmpeg -i {url} -headers "Referer: {ref}" -c copy {output}
 Execute via tokio::process::Command
 Parse stderr for progress (frame=, time=, speed=)
 Send progress updates via channel
 Handle errors gracefully (network, format, permissions)
4. Update Download Router
 Modify 
ipc.rs
 to detect streaming vs direct
 Route streaming URLs to ffmpeg downloader
 Keep existing engine for direct HTTP downloads
 Update filename logic to use container extension
5. Extension Updates
 Update 
getSmartExtension()
 to handle more formats
 Add RTMP, RTSP detection in background.js
 Map more formats: .mpd → .mp4, rtmp:// → .flv
6. UI/UX Polish
 Show "Downloading stream..." status in modal
 Handle "ffmpeg not found" error in frontend
 Display helpful error with install link
 Test with multiple format types
Estimated Effort
Universal ffmpeg Integration: 3-4 hours

Format detection: 30 min
ffmpeg module: 1.5 hours
Router updates: 1 hour
Testing: 1 hour
Native Parser Alternative (NOT recommended): 20-30 hours

Would only handle ONE format (HLS)
Wouldn't solve DASH, RTMP, RTSP, etc.
Dependencies
toml
# For ffmpeg detection
which = "6.0"  # Find executables in PATH
# For ffmpeg execution (already have)
tokio = { version = "1.0", features = ["process"] }
# For progress parsing
regex = "1.10"  # Parse ffmpeg output
Supported Formats Reference
Adaptive Streaming (HTTP-based)
HLS (.m3u8) → Output: 
.ts
MPEG-DASH (.mpd) → Output: .mp4 or .webm
Smooth Streaming (.ism) → Output: .mp4
HDS (.f4m) → Output: .flv
Real-Time Protocols
RTMP/RTMPS (rtmp://, rtmps://) → Output: .flv or .mp4
RTSP/RTSPS (rtsp://, rtsps://) → Output: .mp4
MMS/MMSH (mms://, mmsh://) → Output: .asf or .wmv
SRT (srt://) → Output: 
.ts
 or .mp4
Direct Downloads (Our Engine)
Any direct HTTP(S) file: .mp4, .mkv, .avi, .mov, etc.
Fallback to our engine for better control and progress
Notes
ffmpeg is 100MB uncompressed, but can be stripped to ~30MB for just video functionality
Most users downloading streaming content likely already have ffmpeg installed
This matches how ALL professional tools work: yt-dlp, streamlink, N_m3u8DL-CLI
Future: Bundle ffmpeg with installer or auto-download on first use