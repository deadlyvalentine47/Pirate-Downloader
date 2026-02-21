# Pirate Downloader - MVP4 Product Requirements Document

> [!IMPORTANT]
> **GOAL**: "IDM Turbo Mode & Cleanup"
> **FOCUS**: Speed, Concurrency, Architecture Refinement
> **NOTE**: Move away from sidecars to pure native performance.

## 1. Executive Summary

MVP4 marks the transition from "functional" to "competitive." We have proven we can download cloaked HLS streams that trick standard tools. Now we need to make it as fast as IDM and clean up the legacy code left behind by our FFmpeg experiments.

## 2. Core Objectives

### 🧹 2.1 FFmpeg Decommissioning (Cleanup)
We have successfully implemented a native Rust HLS engine that outperforms FFmpeg's sidecar in reliability. We must now remove the legacy baggage.
*   **Tasks**:
    *   Delete `ffmpeg-x86_64-pc-windows-msvc.exe` from `src-tauri`.
    *   Remove `ffmpeg` from `externalBin` in `tauri.conf.json`.
    *   Remove `shell:allow-execute` and `shell:allow-spawn` permissions related to FFmpeg from `capabilities/default.json`.
    *   Rename `FfmpegStrategy` to `StreamStrategy` or `HlsStrategy` in the codebase.
    *   Remove all FFmpeg-related logging and logic in `lib.rs` and `strategy/ffmpeg.rs`.

### 🚀 2.2 IDM Turbo Mode (Concurrency)
Sequential downloading of 3,000+ segments is too slow. We need massive parallelization.
*   **Features**:
    *   **Concurrent Segment Downloads**: Download up to 10-16 segments simultaneously.
    *   **Ordered Writing**: Ensure segments are appended to the file in the correct order even when downloaded out of sync.
    *   **Connection Pooling**: Reuse HTTP connections effectively to avoid handshake overhead for every segment.
*   **Technical Approach**:
    *   Use a `Semaphore` to limit active downloads.
    *   Use a `FuturesUnordered` or similar stream management to track concurrent tasks.
    *   Maintain a buffer or handle atomic sequential writes to the output file.

### 🛡️ 2.3 Robust HLS Engine (Reliability)
*   **Recursive Manifest Support**: Handle Master Playlists by automatically selecting the highest bandwidth variant.
*   **Advanced Retries**: Implement exponential backoff for failed segments.
*   **Integrity Checks**: Verify that downloaded segments are valid TS chunks (basic header check).

### 📊 2.4 UI Performance Enhancements
*   **Speed Meter**: Real-time Mbps calculation for HLS streams.
*   **Segment Progress**: Visual representation of the segment queue (optional).

## 3. Architecture Changes

*   **`src-tauri/src/core/strategy/hls.rs`**: Rename `ffmpeg.rs` to `hls.rs` and refactor the sequential loop into a concurrent worker pool.
*   **`src-tauri/src/lib.rs`**: Update strategy selection logic.

## 4. Success Metrics
*   **Speed**: HLS download speed should be >80% of the user's available bandwidth.
*   **Cleanliness**: Zero mention of "ffmpeg" in the final binary and config files.
