# Seamless Streaming Plan: Bundling `ffmpeg`

This document outlines a seamless, user-friendly approach to support a wide variety of streaming formats by bundling `ffmpeg` directly within the application. This ensures a "zero-setup" experience for the end-user.

## 1. Core Idea: `ffmpeg` as a Bundled Asset

To provide a superior user experience and compete with established downloaders, we will not require users to install `ffmpeg` manually. Instead, we will treat `ffmpeg` as an internal component of Pirate Downloader.

-   **Zero-Setup:** The user installs our application, and everything they need is included.
-   **Reliability:** The application will know the exact location of the `ffmpeg` executable it was bundled with, eliminating path and version issues.
-   **Leverages Existing Work:** The `DownloadStrategy` refactoring is the perfect foundation for this approach.

## 2. Execution Plan

The plan is divided into distinct, sequential phases.

### **Phase 1: Acquire & Bundle `ffmpeg`**

1.  **Acquire `ffmpeg` Executable:** A pre-compiled, static `ffmpeg.exe` binary will be sourced from a trusted provider (e.g., gyan.dev).
2.  **Integrate into Project:** The `ffmpeg.exe` file will be placed within the `src-tauri` directory.
3.  **Configure Bundling:** `tauri.conf.json` will be modified to include `ffmpeg.exe` as a bundled "resource". This ensures the executable is packaged with the application installer.

### **Phase 2: Backend - The Universal `ffmpeg` Strategy**

1.  **New `FfmpegStrategy`:**
    -   Create `src-tauri/src/core/strategy/ffmpeg.rs`.
    -   This strategy will implement the `DownloadStrategy` trait.

2.  **"Router" Logic in `lib.rs`:**
    -   A detection function, `is_streaming_url()`, will determine if a URL requires `ffmpeg`.
    -   The main `start_download` function will use this detector to choose the correct strategy:
        -   **Stream URL** -> `FfmpegStrategy`
        -   **Direct File URL** -> `HttpStrategy` (our existing engine)

3.  **`FfmpegStrategy` Implementation Details:**
    -   **Path Resolution (No `which`):** The strategy's first step will be to get the path to the bundled executable using Tauri's `app.path().resolve_resource()` API. This replaces the need to check the system PATH.
    -   **Error Handling:** If the resource cannot be found, it indicates a broken installation. The strategy will return a critical internal error. There is no longer a need to prompt the user to install anything.
    -   **Process Execution & Progress Parsing:** The logic to spawn the `ffmpeg` process and parse its `stderr` for progress remains the same as in the previous plan.

### **Phase 3: Frontend - A Truly Seamless Experience**

1.  **Displaying Stream Progress:** The UI will be adapted to handle progress updates from `ffmpeg`, which may be based on time and speed rather than percentage for certain streams (like live broadcasts).
2.  **No More "Not Found" Dialogs:** The user will never be prompted to install `ffmpeg`. A download will either work, or it will fail with a generic error if something unexpected goes wrong with the bundled executable.

This refined plan ensures we deliver a polished, professional, and user-friendly feature that "just works" out of the box.
