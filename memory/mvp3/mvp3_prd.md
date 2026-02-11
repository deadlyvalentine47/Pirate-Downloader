# Pirate Downloader - MVP3 Product Requirements Document

> [!IMPORTANT]
> **GOAL**: "Overthrow IDM"
> **FOCUS**: Integration, Media Sniffing, Reliability
> **NOTE**: Dont break anything but improve it.

## 1. Executive Summary

MVP3 is not about "adding features." It is about **breaking the isolation** of the app.
We are moving from a "Copy-Paste Downloader" to a "Browser Takeover Tool."

## 2. The "IDM Killer" Core Features

### 🔌 2.1 Browser Integration (Priority 1)
**The Hook**: Users should never manually open the app.
*   **Architecture**:
    *   **Extension (Chrome/Firefox)**: Intercepts `activeTab` downloads.
    *   **Native Messaging Host**: A rust binary (sidecar) that bridges Extension <-> Main App.
*   **Features**:
    *   **Context Menu**: "Download with PirateDownloader".
    *   **Click Interception**: Automatically catches `.exe`, `.zip`, `.iso`, `.mp4`, `.ts`, etc.
    *   **Media Sniffer**: Floating button when video stream detected.

#### Technical Spec: Native Messaging Protocol
Communication between Chrome Extension and Rust Host via `stdio`.
**Message Schema (JSON):**
```json
{
  "type": "DOWNLOAD_REQUEST",
  "payload": {
    "url": "https://example.com/file.zip",
    "filename": "file.zip",
    "headers": {
      "Cookie": "session_id=...",
      "User-Agent": "Mozilla/5.0..."
    },
    "referrer": "https://example.com/"
  }
}
```

### 📹 2.2 Media Grabber (Priority 2)
**The Moat**: 50% of users use IDM for video downloading.
*   **Detection**:
    *   Extension monitors network traffic (webRequest API).
    *   Detects `m3u8` playlists and `ts` segments (MIME: `application/vnd.apple.mpegurl`, `video/mp2t`).
*   **Processing**:
    *   **Backend**: Use `m3u8-rs` to parse playlists.
    *   **Download Engine**: Custom `HlsStrategy` in `DownloadEngine`.
        *   Master Playlist -> Select Best Quality (Highest Bandwidth).
        *   Media Playlist -> Download Segments (Concurrent).
        *   Decryption -> Handle AES-128 keys if present.
    *   **Stitching**: Merge segments into `.mp4` using `ffmpeg` (bundled sidecar).

### 🔄 2.3 Smart Link Refresh (Priority 3)
**The Reliability**: "It just works"
*   **Problem**: Links expire (403 Forbidden) during large downloads.
*   **Solution**:
    1.  **Detection**: `DownloadEngine` receives 403 on chunk retry.
    2.  **State Change**: Download State -> `WaitingForLink`.
    3.  **UI Trigger**: Popup "Link Expired. Please visit download page."
    4.  **Extension Logic**: When user visits original page, Extension captures NEW link for SAME file.
    5.  **Resume**: Extension sends `LINK_UPDATE` message -> App updates URL/Cookies -> Resume.

## 3. UI/UX "Power Upgrade"

We need to move from "Clean & Simple" to **"Dense & Informative"**.

### 📊 3.1 The "Grid" View (Power User)
*   **Current**: Single card per download. Too much whitespace.
*   **New**: Dense table/grid with columns:
    *   Name | Size | Speed | Progress | Status | Added | Source
*   **Graph**: Real-time speed graph (canvas based) for dopamine hit.

### 🖥️ 3.2 System Integration
*   **Tray Icon**: App minimizes to tray.
*   **Floating Window**: A small "Drop Zone" that stays on top (optional).

## 4. Technical Strategy

### 4.1 Integration Technology
*   **Native Messaging**:
    *   Use specific `allowed_origins` in manifest.
    *   Rust Host: A separate binary `pirate-host` built from the same workspace.
    *   IPC: `pirate-host` sends data to Main App via TCP (localhost) or Named Pipes.

### 4.2 HLS Engine
*   **Crates**:
    *   `m3u8-rs`: Playlist parsing.
    *   `aes`: Decryption.
*   **FFmpeg**: Bundle `ffmpeg` static binary for final remuxing (safest bet for compatibility).

## 5. Development Phases

### Phase 1: The Bridge
*   **Workspace Restructuring**:
    *   Convert `src-tauri` into a Cargo Workspace.
    *   Member 1: `pirate-app` (The main Tauri app).
    *   Member 2: `pirate-host` (The Native Messaging Host).
    *   Shared Lib: `pirate-core` (Shared types/logic if needed).
*   **Chrome Extension**:
    *   Manifest V3 with `nativeMessaging` permission.
    *   Background script to handle `onDownloadCreated` (simulated interception).
*   **Native Messaging Host (`pirate-host`)**:
    *   Standard Input/Output (stdio) loop.
    *   Protocol: Length-prefixed JSON (Chrome Native Messaging standard).
    *   Installer: Script to register the host manifest (Registry on Windows).
*   **Pipeline**:
    *   Extension sends "Hello" -> Host echoes -> App logs event.

### Phase 2: The Interceptor
*   Extension intercepts generic downloads (`onDeterminingFilename`).
*   App receives URL + Cookies + UserAgent.
*   `DownloadEngine` supports "Cookies" header (currently missing).

### Phase 3: The Sniffer (HLS)
*   **Packet Analysis**:
    *   Extension monitors `onBeforeRequest` for `.m3u8` and `.ts` MIME types.
    *   Captures `REFERER` and `COOKIE` essential for HLS.
*   **Core Implementation**:
    *   `src-tauri/src/core/hls.rs`: New module using `m3u8-rs`.
    *   Logic: Master Playlist -> Variety Selection -> Media Playlist -> Segment Queue.
*   **Stitching**:
    *   Download segments to temp folder.
    *   Use `ffmpeg` sidecar to merge: `ffmpeg -i list.txt -c copy output.mp4`.
    *   (Future: Native Rust muxing if feasible, but FFmpeg is MVP safe bet).

### Phase 4: The Polymerization
*   Combine everything into "Power UI".
