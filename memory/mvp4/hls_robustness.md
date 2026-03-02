# Feature Specification: High-Performance HLS Robustness

## 1. Objective
Transform the current basic HLS segment-by-segment downloader into a "Peak Efficiency" engine capable of handling complex HLS streams, master playlists, and adaptive bitrate selection with a focus on speed and reliability.

## 2. Core Requirements

### 2.1 Master Playlist Parsing (The "Smart" Resolver)
*   **Recursive Manifest Discovery:** The engine must recognize when a URL is a Master Playlist and recursively fetch all variant playlists (Sub-manifests).
*   **Variant Analysis:** Extract metadata from variants, including:
    *   `BANDWIDTH`: To estimate file size.
    *   `RESOLUTION`: To allow user selection (e.g., 1080p, 720p).
    *   `CODECS`: To ensure compatibility.
*   **Default Selection:** Automatically select the highest quality stream if the user has not specified a preference.

### 2.2 Parallel Segment Downloading (Multi-threaded HLS)
*   **Worker Pool Integration:** Unlike the current sequential HLS download, the engine should use a worker pool (similar to the `HttpStrategy`) to download multiple segments simultaneously.
*   **Dynamic Queue Management:** As the manifest is parsed, segment URLs should be added to a thread-safe queue.
*   **Window-based Prefetching:** Implement a sliding window to download segments in order but in parallel (e.g., downloading segments 1-10 at once, then 11-20).

### 2.3 Advanced Stream Features
*   **AES-128 Decryption Support:** Automatically detect `#EXT-X-KEY` tags, fetch the decryption key, and decrypt segments on-the-fly using AES-128-CBC.
*   **Discontinuity Handling:** Properly manage `#EXT-X-DISCONTINUITY` tags to ensure seamless merging of segments even if they have different timestamps or encodings.
*   **Byte-Range Support:** Handle manifests that use `#EXT-X-BYTERANGE` to fetch segments from within a single large file.

### 2.4 Reliability & Efficiency
*   **Sparse Allocation (Pre-sizing):** Use bandwidth metadata to estimate total size and pre-allocate disk space to minimize fragmentation.
*   **Adaptive Retries:** Implement per-segment retry logic with exponential backoff.
*   **Chunk Reuse:** If a segment download fails, only that segment should be retried, not the entire stream.
*   **Zero-Copy Appending:** Use efficient buffered writing to append segments to the final file without unnecessary memory copies.

## 3. Technical Architecture

### 3.1 Backend (Rust)
*   **`HlsStrategy` Refactor:** Move from the current `hls.rs` logic to a new `HlsManager` that coordinates a pool of `HlsWorker` threads.
*   **Manifest Parser:** Use a specialized crate (like `m3u8-rs`) or a robust custom parser for high-speed manifest analysis.
*   **State Machine:** Add `WaitingForQualitySelection` state to `DownloadState` to allow the UI to prompt the user for resolution.

### 3.2 Frontend (React)
*   **Quality Selector UI:** When a master playlist is detected, show a modal with available resolutions, bitrates, and estimated sizes.
*   **Segment Progress Bar:** A detailed progress visualization showing the status of individual HLS segments (similar to the "blocks" view in classic download managers).

## 4. Performance Targets
*   **Concurrency:** Support up to 32 concurrent segment downloads.
*   **Memory:** Maintain a low memory footprint (< 50MB) even for 4K streams by using streaming writes.
*   **Overhead:** Minimize manifest parsing time to < 100ms.
