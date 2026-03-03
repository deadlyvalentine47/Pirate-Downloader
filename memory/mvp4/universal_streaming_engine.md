# Universal Streaming Engine - Detailed Design

The **Universal Streaming Engine** is a modular, feature-toggle-based architecture designed to handle HLS, DASH, and Platform-Specific (YouTube/Vimeo) streams with peak efficiency.

## 1. Core Architecture
The engine is split into four distinct layers to ensure maintainability and extensibility.

### A. The Strategy Hub (`UniversalStreamingStrategy`)
The main entry point that implements the `DownloadStrategy` trait. It holds the `StreamingConfig`.
- **Responsibility:** Orchestrates the flow between Resolver, Downloader, and Processor.
- **Config Toggles:**
    - `enable_parallel_segments`: (Default: True)
    - `enable_header_stripping`: (Default: True) - The ".jpg" fix.
    - `enable_platform_resolvers`: (Default: True) - YouTube/Vimeo support.

### B. The Resolver Layer (`stream/resolver.rs`)
The "Brain" that identifies the stream type and extracts the source units.
- **HLS Resolver:** Parses `.m3u8` and returns a list of Segment URLs.
- **DASH Resolver:** Parses `.mpd` and returns segment templates.
- **Platform Resolver (YouTube):** Uses specialized logic to extract direct video/audio URLs.

### C. The Parallel Downloader (`stream/downloader.rs`)
A high-performance worker pool optimized for small, dynamic segments.
- **Concurrency:** Uses `futures_util::stream::StreamExt::buffered(16)` to download up to 16 segments simultaneously.
- **Retry Logic:** Individual segment retries (3 attempts) before failing the whole download.

### D. The Processor Layer (`stream/processor.rs`)
The "Cleaner" that ensures data integrity before it hits the disk.
- **MPEG-TS Sync Detection:** Scans the first 1024 bytes of any segment for the `0x47` sync byte.
- **Header Stripping:** Discards any "fake" image headers (JPG/PNG) found before the valid video data.

## 2. Peak Efficiency & Safety Features

### A. Connection Persistence (TCP/TLS Reuse)
- **Mechanism:** Uses a single, shared `reqwest::Client` with a persistent connection pool.
- **Benefit:** Eliminates the 200ms+ handshake delay for every 2-second segment, maximizing bandwidth saturation.

### B. Bounded Backpressure (Memory Safety)
- **Mechanism:** Implements a "High-Water Mark" buffer (Default: 32 segments or 128MB).
- **Benefit:** If the disk writer is slower than the internet (e.g., slow USB drive), the downloader will pause once the buffer is full, preventing the app from crashing due to RAM exhaustion.
- **Zero Speed Impact:** In normal conditions (Disk > Internet), the buffer never fills, and the network runs at 100% speed.

### C. Progressive Sequential Flushing
- **Mechanism:** A dedicated writer loop that flushes segments to disk the moment they are available in the correct sequence.
- **Benefit:** Minimizes the time data sits in RAM and ensures "Smooth" disk activity rather than "Spiky" writes.

### D. Index-based Resumption
- **Mechanism:** Tracks the last successfully written segment index in a `.progress` (JSON) sidecar file.
- **Benefit:** Allows users to pause and resume multi-hour streams (like 4K movies or live stream archives) without restarting from the beginning.

## 3. Decision Routing Logic
The app decides which strategy to use in `src-tauri/src/lib.rs` based on the URL and initial headers:

| URL/Header Pattern | Strategy Selected |
| :--- | :--- |
| `youtube.com`, `youtu.be` | `UniversalStreamingStrategy` |
| `.m3u8`, `application/vnd.apple.mpegurl` | `UniversalStreamingStrategy` |
| `.mpd`, `application/dash+xml` | `UniversalStreamingStrategy` |
| Everything Else (with `Content-Length`) | `HttpStrategy` (Chunked) |

## 4. Implementation Phases
1.  **Phase 1: Foundation.** Create folder structure and the `UniversalStreamingStrategy` hub with shared connection pool.
2.  **Phase 2: Parallel Downloader.** Implement the multi-threaded fetcher with backpressure and sequential flushing.
3.  **Phase 4: Robustness.** Add the MPEG-TS Sync Detection and Header Stripping logic.
4.  **Phase 5: Platform Expansion.** Add the YouTube Platform Resolver.
