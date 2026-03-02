# Pirate Downloader: Project Summary & Capabilities

## 1. Overview
Pirate Downloader is a high-performance, multi-threaded download manager built with **Tauri v2**, **React 19**, and **Rust**. It is designed to bridge the gap between simple browser downloads and professional-grade tools like IDM, offering native HLS/Dash support and browser extension integration.

## 2. Core Capabilities

### 2.1 Multi-threaded HTTP Downloads (`HttpStrategy`)
*   **Segmented Downloads:** Splits a single file into multiple chunks (default 16 threads) and downloads them in parallel.
*   **Sparse File Allocation:** Pre-allocates disk space on Windows using sparse files to prevent disk fragmentation and ensure immediate availability of the target file.
*   **Adaptive Retries:** Automatically retries failed chunks with exponential backoff and speed enforcement.
*   **Speed Monitoring:** Real-time speed calculation and progress reporting to the frontend via Tauri events.

### 2.2 Native HLS/Streaming Downloads (`HlsStrategy`)
*   **Custom Rust Engine:** Unlike other downloaders, it does **not** rely on the FFmpeg binary. It implements a native Rust HLS parser and downloader.
*   **Manifest Parsing:** Fetches and parses `.m3u8` playlists, resolves segment URLs (relative and absolute), and appends bytes sequentially to a single output file.
*   **Protocol Support:** Detects HLS (`.m3u8`), DASH (`.mpd`), and other streaming protocols automatically.
*   **Automatic Container Detection:** Detects if a stream should be saved as `.ts` or `.mp4` based on the manifest content.

### 2.3 Browser Extension & Native Integration
*   **Native Messaging Host (`pirate-host`):** A specialized Rust bridge that allows a Chrome/Firefox extension to communicate with the desktop app.
*   **One-Click Interception:** The browser extension intercepts standard browser downloads and offloads them to Pirate Downloader.
*   **Media Sniffer:** Automatically detects HLS streams on pages (like video players) and offers a "Download with Pirate" option.
*   **Context Menu:** Right-click any link, image, or video to send it directly to the downloader.

### 2.4 "IDM Mode" Link Refreshing
*   **Link Expiry Handling:** Detects `403 Forbidden` errors (common with expired download links).
*   **WaitingForLink State:** Transitions the download to a special state and signals the browser extension to watch for a "refreshed" link when the user visits the page again.
*   **Auto-Resume:** Automatically updates the URL and resumes the download once a new link is captured.

## 3. Technical Architecture

*   **Frontend:** React 19 (TypeScript), Vite, Zustand for state management, Tailwind-inspired CSS for a modern "Pirate" aesthetic.
*   **Backend:** Rust (Tauri v2), `tokio` for async concurrency, `reqwest` for HTTP operations.
*   **IPC:** Local Sockets (Pipes on Windows, Unix Sockets on Unix) using the `interprocess` crate for low-latency communication between the host bridge and the main app.
*   **Persistence:** `rusqlite` (SQLite) for storing download history and `serde` for serializing active download states.

## 4. Current State
*   **Build Status:** Production-ready installers (MSI, EXE) are generated successfully.
*   **Stability:** Implemented timeouts and robust error logging for all network operations.
*   **HLS Logic:** Functional for single-playlist HLS streams; multi-threaded segment downloading and master playlist selection are slated for the next phase.
