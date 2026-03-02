# Pirate Downloader - Project Context

Pirate Downloader is a high-performance, multi-threaded download manager built with **Tauri v2**, **React**, and **Rust**. It features a browser extension integration for capturing downloads and refreshing expired links.

## Project Overview

*   **Frontend:** React 19 (TypeScript) powered by Vite.
*   **Backend:** Rust (Tauri v2).
*   **Architecture:**
    *   **Tauri App:** The core application containing the download engine, UI, and IPC server.
    *   **Native Host (`pirate-host`):** A lightweight Rust bridge between the Browser Extension and the Tauri App. It receives Native Messaging from Chrome/Firefox and forwards it to the Tauri App via Local Sockets (IPC).
    *   **Browser Extension:** A Manifest V3 extension that intercepts browser downloads, sniffs for HLS/DASH streams, and provides a context menu for quick downloads.
    *   **Shared Library (`pirate-shared`):** A Rust crate defining common types (`IpcMessage`, `DownloadRequest`) and IPC constants used across the ecosystem.

## Tech Stack

*   **Framework:** Tauri v2
*   **State Management:** Zustand (Frontend), Custom Download Manager (Backend Rust).
*   **HTTP Client:** `reqwest` (Rust) for both multi-threaded chunked downloads and streaming.
*   **Streaming Strategy:** Custom HLS/Dash downloader (in `hls.rs`). It parses manifests, resolves segment URLs, and appends bytes manually using `reqwest` and `tokio::fs`.
*   **IPC:** `interprocess` for Local Sockets (Pipes on Windows, Unix Sockets on macOS/Linux).
*   **Persistence:** `rusqlite` for download history and `serde` for state serialization.

## Key Features & Logic

*   **Multi-threaded Downloads:** Uses `HttpStrategy` with worker threads. Supports adaptive retries and speed enforcement.
*   **Sparse File Allocation:** Uses `filesystem::allocate_sparse_file` to pre-allocate disk space, preventing fragmentation.
*   **Link Refresh (IDM Mode):** Detects `403 Forbidden` errors, transitions to `WaitingForLink` state, and updates the URL via the browser extension's link sniffer.
*   **Native Messaging:** The extension communicates with `pirate-host.exe` using standard length-prefixed JSON.
*   **Integrity Checks:** Verifies byte count and chunk completion before marking a download as finished.

## Building and Running

### Prerequisites
*   Rust (1.75+)
*   Node.js (v20+)

### Development
1.  **Install Frontend Deps:** `npm install`
2.  **Run Tauri App:** `npm run tauri dev`
3.  **Build Native Host:**
    ```powershell
    cd src-tauri
    cargo build -p pirate-host
    ```
4.  **Register Extension Host:** Run `register_host.bat` as Administrator.
5.  **Load Extension:** Load the `/extension` folder as an unpacked extension in Chrome/Edge.

## Project Structure

*   `/src`: React components, hooks, and Zustand stores.
*   `/extension`: Manifest V3 service worker (`background.js`) and UI.
*   `/src-tauri/src/core`:
    *   `engine.rs`: Orchestrates download strategies.
    *   `strategy/http.rs`: Multi-threaded chunked download logic.
    *   `strategy/hls.rs`: Native HLS/Dash segment downloader.
    *   `state.rs`: Download state machine and metadata types.
*   `/src-tauri/host`: Native messaging host implementation.
*   `/src-tauri/shared`: Shared IPC protocols.

## Development Conventions

*   **Async Rust:** Extensively uses `tokio` for concurrency and `async-trait` for strategies.
*   **Error Handling:** Custom `DownloadError` enum in `core/error.rs`.
*   **Logging:** `tracing` crate with structured logging.
*   **IPC Protocol:** JSON-over-Socket using `IpcMessage` enum.
