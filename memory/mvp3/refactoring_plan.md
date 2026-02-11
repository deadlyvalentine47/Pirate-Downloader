# Refactoring Plan: MVP3 Readiness

> [!IMPORTANT]
> **GOAL**: Extract `run_download_task` from `lib.rs` and modularize it to support future HLS/Stream downloads.
> **ESTIMATED TIME**: 1-2 Days

## 1. Backend Refactoring (`src-tauri`)

### 1.1 Create `core/engine.rs` (The Orchestrator)
Currently, `lib.rs` manages the download loop. We will move this to a dedicated struct.

```rust
// src-tauri/src/core/engine.rs

pub struct DownloadEngine;

impl DownloadEngine {
    pub async fn start(
        metadata: DownloadMetadata,
        control: Arc<DownloadControl>,
        manager: DownloadManager
    ) -> Result<DownloadCommandResult, DownloadError> {
        // Logic from run_download_task goes here
    }
}
```

### 1.2 Implement Strategy Pattern (The Extensibility)
To support HLS (m3u8) later, we need a trait that defines "How to download".

```rust
// src-tauri/src/core/strategy.rs

pub trait DownloadStrategy {
    async fn execute(&self, ctx: &Context) -> Result<(), DownloadError>;
}

// Implementations:
// 1. HttpStrategy (Current Range-based download)
// 2. HlsStrategy (Future m3u8 download)
```

### 1.3 Clean up `lib.rs`
`lib.rs` should ONLY contain Tauri commands that delegate to the Engine.

## 2. Frontend Refactoring (`src`)

### 2.1 Remove Inline Styles
Files like `ActionButtons.tsx` use `style={{ ... }}`. These must be converted to Tailwind classes.
*   `ActionButtons.tsx`
*   `DownloadStatus.tsx`
*   `App.tsx`

### 2.2 Component Modularity
*   Move "Confirm Dialog" logic out of `ActionButtons` into a custom hook `useConfirmation`.

## 3. Execution Steps

1.  **Backend Strategy**: Create `core/engine.rs` and extract `run_download_task`.
2.  **Backend Dependency**: Resolve imports in new module.
3.  **Backend Integration**: Integrate `DownloadEngine` back into `lib.rs`.
4.  **Backend Verification**: Verify existing functionality (Download/Pause/Resume).
5.  **Frontend Cleanup**: Refactor identified components to Tailwind.

