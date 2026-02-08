# Download Control Implementation Plan

**Feature**: Pause/Resume/Stop/Cancel Downloads  
**Started**: 2026-02-08  
**Status**: Planning  

---

## Overview

Implement 4 download control operations with proper state management:
1. **Pause** - Temporarily halt, preserve state
2. **Resume** - Continue from last position
3. **Stop** - Graceful shutdown, keep partial file
4. **Cancel** - Terminate and cleanup everything

---

## Implementation Phases

### Phase 1: State Management (Backend)
**Files to Create/Modify**:
- `src-tauri/src/core/state.rs` - Download state enum and structures
- `src-tauri/src/core/persistence.rs` - Save/load state to JSON
- `src-tauri/src/lib.rs` - Add new Tauri commands

**Tasks**:
1. Define `DownloadState` enum (Pending, Active, Paused, Stopped, Completed, Failed, Cancelled)
2. Define `DownloadMetadata` struct (URL, filepath, size, chunks, timestamps)
3. Implement state serialization/deserialization
4. Add state file management (.part.state files)

### Phase 2: Control Commands (Backend)
**Files to Modify**:
- `src-tauri/src/lib.rs` - Add pause/resume/stop/cancel commands

**Tasks**:
1. Implement `pause_download()` command
2. Implement `resume_download()` command
3. Implement `stop_download()` command
4. Implement `cancel_download()` command
5. Add proper error handling for each command
6. Add comprehensive logging (DEBUG, INFO, WARN levels)

### Phase 3: Download Engine Integration
**Files to Modify**:
- `src-tauri/src/lib.rs` - Modify download_file to support state

**Tasks**:
1. Add state tracking to download loop
2. Check for pause/stop signals during download
3. Save state periodically during download
4. Handle graceful shutdown on stop
5. Cleanup on cancel

### Phase 4: Frontend Integration
**Files to Create/Modify**:
- `src/stores/downloadStore.ts` - Add state management
- `src/components/download/DownloadControls.tsx` - Add control buttons
- `src/hooks/useDownload.ts` - Add control operations

**Tasks**:
1. Add state to Zustand store
2. Create Pause/Stop/Cancel buttons for active downloads
3. Create Resume/Cancel buttons for paused downloads
4. Add confirmation dialog for Cancel
5. Add visual state indicators (badges)

---

## State File Format

```json
{
  "url": "https://example.com/file.zip",
  "filepath": "C:/Downloads/file.zip",
  "total_size": 1073741824,
  "downloaded_bytes": 536870912,
  "state": "paused",
  "thread_count": 16,
  "completed_chunks": [0, 1, 2, 3, 4, 5, 6, 7],
  "incomplete_chunks": [8, 9, 10, 11, 12, 13, 14, 15],
  "created_at": "2026-02-08T12:30:00Z",
  "paused_at": "2026-02-08T12:35:00Z",
  "resumed_at": null
}
```

---

## Logging Strategy

All operations will include structured logging:

```rust
// Pause
info!(url = %url, downloaded_bytes, "Download paused by user");

// Resume
info!(url = %url, remaining_chunks = incomplete.len(), "Resuming download");

// Stop
info!(url = %url, "Download stopped gracefully");

// Cancel
warn!(url = %url, "Download cancelled - cleaning up files");
```

---

## Testing Checklist

- [ ] Pause mid-download, verify state saved
- [ ] Resume paused download, verify continues correctly
- [ ] Stop download, verify .part file preserved
- [ ] Resume stopped download after app restart
- [ ] Cancel from active state, verify cleanup
- [ ] Cancel from paused state, verify cleanup
- [ ] Cancel from stopped state, verify cleanup
- [ ] Verify no data loss on pause/resume
- [ ] Verify state persistence across restarts

---

## Next Steps

1. Start with Phase 1: Create state management structures
2. Add logging infrastructure to all operations
3. Implement backend commands
4. Test backend thoroughly
5. Add frontend controls
