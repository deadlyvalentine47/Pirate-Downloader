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

### 📹 2.2 Media Grabber (Priority 2)
**The Moat**: 50% of users use IDM for video downloading.
*   **Detection**:
    *   Extension monitors network traffic (webRequest API).
    *   Detects `m3u8` playlists and `ts` segments.
*   **Processing**:
    *   **Backend**: Need `ffmpeg` sidecar or Rust `m3u8-rs` + `reqwest` to download segments.
    *   **Stitching**: Merge segments into `.mp4` automatically.

### 🔄 2.3 Smart Link Refresh (Priority 3)
**The Reliability**: "It just works"
*   **Problem**: Links expire (403 Forbidden) during large downloads.
*   **Solution**:
    1.  Detect 403 error on chunk failure.
    2.  Pause download.
    3.  Popup: "Link Expired. Please visit download page."
    4.  Extension captures NEW link for SAME file.
    5.  Resume download with new authentication cookies.

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
*   **Native Messaging**: Use `tauri-plugin-native-messaging` (if available) or raw stdin/stdout handling in a separate binary.
*   **Protocol**: JSON-RPC over stdio.

### 4.2 HLS Engine
*   **Library**: `hls_downloader` crate or custom impl using `m3u8-rs`.
*   **FFmpeg**: Bundle `ffmpeg` static binary for final remuxing (safest bet for compatibility).

## 5. Development Phases

### Phase 1: The Bridge (Week 1-2)
*   Build minimal Chrome Extension.
*   Implement Native Messaging in Rust.
*   Send "Hello World" from Chrome to PirateDownloader.

### Phase 2: The Interceptor (Week 3)
*   Extension intercepts generic downloads.
*   App receives URL + Cookies + UserAgent.

### Phase 3: The Sniffer (Week 4-5)
*   Extension detects `.m3u8`.
*   App implements basic HLS downloader.

### Phase 4: The Polymerization (Week 6)
*   Combine everything into "Power UI".
