# Codebase Assessment Report
**Date**: 2026-02-11
**Focus**: Readiness for MVP3 (Browser Integration & Media Sniffing)

## 1. Executive Summary
The codebase is **functional but fragile**.
While we claimed to have "refactored into modules" in MVP2 Phase 0, the truth is that `lib.rs` still contains the massive `run_download_task` function (300+ lines) which handles:
1.  Worker spawning
2.  Chunk delegation
3.  Retry logic
4.  Speed enforcement
5.  State management synchronization

**Risk**: Adding "HLS Streaming" or "Browser Integration" to this single function will make it unmaintainable.

## 2. Backend Analysis (`src-tauri`)

### 🚩 Critical Issues
*   **`lib.rs` is acting as a God Object**: It shouldn't contain `run_download_task`. This logic belongs in `core/engine.rs` or `core/orchestrator.rs`.
*   **Missing Abstractions**: "Download" is currently just a HTTP GET range loop. MVP3 requires different *strategies* (e.g., `HttpStrategy`, `HlsStrategy`).
*   **Tight Coupling**: `run_download_task` is tightly coupled to `reqwest` and filesystem operations. Hard to mock/test.

### ✅ Good Points
*   **`commands/download_control.rs`**: Clean separation of command handling.
*   **`core/types.rs`**: Good shared type definitions.
*   **`utils/logger.rs`**: Solid logging setup.

## 3. Frontend Analysis (`src`)

### ⚠️ Observations
*   **Component Granularity**: Components like `ActionButtons` are doing too much (handling API calls + UI state + Confirmation Dialogs).
*   **State Management**: `downloadStore` is simple, but might need to handle high-frequency updates (60fps speed graphs) carefully.
*   **Styling**: Inline styles (`style={{ ... }}`) are used in many places instead of Tailwind classes. This will make "Power UI" theming hard.

## 4. Refactoring Plan (Pre-MVP3)

Before adding new features, we **MUST** refactor `lib.rs`.

### Task 1: Refactor `run_download_task`
*   Move `run_download_task` to `src-tauri/src/core/download_engine.rs`.
*   Create a `DownloadStrategy` trait:
    ```rust
    trait DownloadStrategy {
        async fn execute(&self, metadata: &Metadata, control: &Control) -> Result<...>;
    }
    ```
*   Implement `StandardHttpStrategy` (current logic).
*   (Future) Implement `HlsStrategy`.

### Task 2: Frontend Cleanup
*   Move all inline styles to Tailwind classes.
*   Extract API calls from components into `src/services/api.ts`.

## 5. Recommendation
**Proceed with Refactoring Phase** before starting MVP3 Phase 1.
Est. Time: 1-2 Days.
