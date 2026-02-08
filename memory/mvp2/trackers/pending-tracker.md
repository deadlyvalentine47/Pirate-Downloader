# MVP2 Pending Tasks Tracker

**Last Updated**: 2026-02-08  
**Status**: Phase 0 Complete ✅ - Moving to Phase 1

---

## Legend

- `[ ]` - Pending
- `[/]` - In Progress
- `[x]` - Completed
- `[!]` - Blocked

**Tags**:
- `#critical` - Blocks other work
- `#backend` - Rust/Tauri work
- `#frontend` - React/TypeScript work
- `#testing` - Test-related
- `#docs` - Documentation
- `#infra` - Infrastructure/tooling

---

## Phase 0: Prerequisites (Foundation)

### Codebase Refactoring
- [x] ~~Create modular file structure~~ `#critical` `#backend` **COMPLETED 2026-02-06**
  - [x] Created `core/` module (types, integrity)
  - [x] Created `network/` module (client, headers)
  - [x] Created `utils/` module (filesystem)
  - [x] Refactored `lib.rs` (333→298 lines)
  - [x] Added module documentation (rustdoc)
  - [x] Verified with 3 large downloads (1.87GB, 3.40GB, 7.05GB)
  - [x] Performance maintained (15-21 MB/s, 100% completion)

### Testing Framework
- [ ] Set up testing infrastructure `#critical` `#testing`
  - [ ] Add test dependencies (mockito, criterion, tempfile)
  - [ ] Create `tests/integration/` directory
  - [ ] Create mock HTTP server for testing
  - [ ] Write unit tests for core modules (80% coverage goal)
  - [ ] Write integration tests (download, pause, resume)
  - [ ] Set up benchmarking with Criterion

### Logging & Observability
- [x] Implement structured logging `#backend` `#infra` **COMPLETED 2026-02-08**
  - [x] Add tracing dependencies
  - [x] Replace all `println!` with `tracing` macros
  - [x] Set up log file rotation
  - [x] Configure log levels (dev: DEBUG, prod: INFO)
- [x] Implement error handling `#backend` `#infra` **COMPLETED 2026-02-08**
  - [x] Create custom error types with thiserror
  - [x] Replace all `.unwrap()` calls
  - [x] Update function signatures to return `Result<T, DownloadError>`

### CI/CD Pipeline
- [ ] Set up GitHub Actions `#critical` `#infra`
  - [ ] Create `.github/workflows/ci.yml`
  - [ ] Configure cross-platform testing (Windows, macOS, Linux)
  - [ ] Add clippy linting
  - [ ] Add rustfmt checks
  - [ ] Set up code coverage reporting (Codecov)
  - [ ] Add security audit (`cargo audit`)

### Dependency Management
- [x] Add MVP2 dependencies `#backend` **COMPLETED 2026-02-08**
  - [x] Add rusqlite for SQLite (v0.32)
  - [x] Add serde/serde_json for serialization (v1)
  - [x] Add chrono for timestamps (v0.4)
  - [x] Add uuid for unique IDs (v1.11)
  - [x] Add tracing/tracing-subscriber for logging (v0.1/v0.3)
  - [x] Add arboard for clipboard (v3.4)
  - [x] Add notify-rust for notifications (v4.11)
  - [x] Add thiserror for error handling (v1.0)
  - [x] Run `cargo audit` for security check (1 vulnerability in `time` crate - needs `tracing-appender` update)

### Frontend Refactoring
- [x] Restructure frontend `#critical` `#frontend` **COMPLETED 2026-02-08**
  - [x] Create component library (5 components: ProgressBar, HistoryItem, HistoryList, DownloadStatus, DownloadControls)
  - [x] Set up Zustand for state management (v4.5.5)
  - [x] Create stores (downloadStore, historyStore)
  - [x] Create custom hooks (useDownload, useTauriEvents)
  - [x] Create utility functions (formatBytes, formatSpeed, formatTime, calculatePercentage)
  - [x] All components in TypeScript
  - [x] App.tsx reduced from 183 to 42 lines (77% reduction)
  - [x] Verified: Full download test passed (7.2GB file, 100% integrity)

### Error Handling
- [x] Implement error handling strategy `#backend` **COMPLETED 2026-02-08**
  - [x] Create `core/error.rs` with custom error types
  - [x] Add thiserror dependency
  - [x] Define DownloadError enum
  - [x] Update all modules to use Result<T, DownloadError>
  - [x] Add error context to all error paths

### Documentation
- [ ] Document codebase `#docs`
  - [ ] Add rustdoc comments to all public APIs
  - [ ] Create CONTRIBUTING.md
  - [ ] Update README.md with new structure
  - [ ] Document module architecture

---

## Phase 1: Essential Features (Priority 1)

### 1. Download Control (Pause/Resume/Stop/Cancel)
- [x] ~~Backend implementation~~ `#backend` `#critical` **COMPLETED 2026-02-08**
  - [x] Design state structure (active, paused, stopped, completed, failed, cancelled)
  - [x] Implement Pause command (save state, halt workers)
  - [x] Implement Resume command (load state, rebuild chunk queue) *Note: Resume updates state but doesn't restart download yet*
  - [x] Implement Stop command (graceful shutdown, keep .part file)
  - [x] Implement Cancel command (terminate, cleanup files + state)
  - [x] Save download state to JSON (.part.state file)
  - [x] Store metadata (URL, filepath, total size, completed bytes, thread count, timestamps)
  - [x] Track completed chunks list
  - [x] Load state on app restart
  - [x] Verify partial file integrity on resume
  - [x] Handle state transitions (pending → active → paused/stopped/cancelled)
  - [x] Integrated control signals into download loop (workers check signals before each chunk)
  - [x] Added DownloadManager for tracking active downloads
  - [x] Added DownloadControl with AtomicU8 signals (0=run, 1=pause, 2=stop, 3=cancel)
- [x] ~~Frontend implementation~~ `#frontend` **COMPLETED 2026-02-08**
  - [x] Add Pause/Stop/Cancel buttons to active downloads
  - [x] Add Resume/Cancel buttons to paused downloads
  - [x] Add Resume/Cancel buttons to stopped downloads
  - [x] Add Retry/Cancel buttons to failed downloads
  - [x] Show confirmation dialog for Cancel (destructive)
  - [x] Update progress bar on resume
  - [x] Show current state visually (badges/colors)
  - [x] Display state-appropriate controls
- [x] ~~Testing~~ `#testing` **COMPLETED 2026-02-08**
  - [x] Test pause mid-download
  - [x] Test resume after pause
  - [x] Test resume after app restart
  - [x] Test stop and resume later
  - [x] Test cancel from all states (active, paused, stopped, failed)
  - [x] Test no data loss on pause/resume
  - [x] Test .part file cleanup on cancel
  - [x] Test state persistence across restarts

### 2. Download Queue Management
- [ ] Backend implementation `#backend`
  - [ ] Create queue data structure
  - [ ] Implement add/remove/reorder operations
  - [ ] Implement concurrency limiting
  - [ ] Auto-start next download on completion
  - [ ] Persist queue to disk
- [ ] Frontend implementation `#frontend`
  - [ ] Create QueueView component
  - [ ] Add drag-to-reorder functionality
  - [ ] Add Start/Stop/Remove buttons
  - [ ] Add global Pause All/Resume All buttons
- [ ] Testing `#testing`
  - [ ] Test queue persistence
  - [ ] Test concurrency limits
  - [ ] Test auto-start behavior

### 3. Automatic Filename Detection
- [ ] Backend implementation `#backend`
  - [ ] Parse Content-Disposition header
  - [ ] Fallback to URL path extraction
  - [ ] Sanitize filenames
  - [ ] Handle duplicate filenames
- [ ] Frontend implementation `#frontend`
  - [ ] Show filename preview
  - [ ] Allow user to edit filename
- [ ] Testing `#testing`
  - [ ] Test header parsing
  - [ ] Test URL extraction
  - [ ] Test duplicate handling

### 3.5. Improve Link Detection & Filename Extraction
- [ ] Backend implementation `#backend` `#enhancement`
  - [ ] Detect streaming links (m3u8, ts segments)
  - [ ] Extract proper filenames from Content-Disposition headers
  - [ ] Handle worker/CDN URLs with encoded filenames
  - [ ] Fallback to content-type based extensions (.ts, .mp4, .mkv)
  - [ ] Add MIME type detection for unknown extensions
  - [ ] Handle redirect chains to get final filename
- [ ] Testing `#testing`
  - [ ] Test m3u8 streaming links
  - [ ] Test worker URLs (cloudflare workers, etc)
  - [ ] Test various video formats (.ts, .mp4, .mkv, .avi)
  - [ ] Test links without extensions
- [ ] Notes
  - Issue: Links like `https://67streams.online/movie/.../index.m3u8` default to `download.dat`
  - IDM detects these as video files and uses proper extensions
  - Need to improve Content-Disposition parsing and MIME type detection

### 4. Download History
- [ ] Backend implementation `#backend`
  - [ ] Create SQLite database
  - [ ] Create history table schema
  - [ ] Implement CRUD operations
  - [ ] Add search functionality
  - [ ] Add date range filtering
- [ ] Frontend implementation `#frontend`
  - [ ] Create HistoryView component
  - [ ] Add search bar
  - [ ] Add date range filter
  - [ ] Add "Open file location" button
  - [ ] Add "Re-download" button
  - [ ] Add "Clear history" button
- [ ] Testing `#testing`
  - [ ] Test history persistence
  - [ ] Test search functionality
  - [ ] Test re-download from history

### 5. Settings & Configuration
- [ ] Backend implementation `#backend`
  - [ ] Create settings table schema
  - [ ] Implement settings CRUD
  - [ ] Add default settings
  - [ ] Validate settings on save
- [ ] Frontend implementation `#frontend`
  - [ ] Create SettingsPanel component
  - [ ] Add General settings section
  - [ ] Add Network settings section
  - [ ] Add UI settings section
  - [ ] Add Advanced settings section
  - [ ] Add validation for inputs
- [ ] Testing `#testing`
  - [ ] Test settings persistence
  - [ ] Test validation
  - [ ] Test settings apply immediately

---

## Phase 2: Power User Features (Priority 2)

### 6. Bandwidth Limiter
- [ ] Backend implementation `#backend`
- [ ] Frontend implementation `#frontend`
- [ ] Testing `#testing`

### 7. Browser Integration
- [ ] Chrome extension `#frontend`
- [ ] Firefox extension `#frontend`
- [ ] Native messaging setup `#backend`
- [ ] Testing `#testing`

### 8. Download Scheduling
- [ ] Backend implementation `#backend`
- [ ] Frontend implementation `#frontend`
- [ ] Testing `#testing`

### 9. Categories & Organization
- [ ] Backend implementation `#backend`
- [ ] Frontend implementation `#frontend`
- [ ] Testing `#testing`

### 10. Clipboard Monitoring
- [ ] Backend implementation `#backend`
- [ ] Frontend implementation `#frontend`
- [ ] Testing `#testing`

---

## Phase 3: UX Enhancements (Priority 3)

### 11. System Tray Integration
- [ ] Backend implementation `#backend`
- [ ] Testing `#testing`

### 12. Desktop Notifications
- [ ] Backend implementation `#backend`
- [ ] Testing `#testing`

### 13. Drag & Drop Support
- [ ] Frontend implementation `#frontend`
- [ ] Testing `#testing`

### 14. Download Verification (Checksums)
- [ ] Backend implementation `#backend`
- [ ] Frontend implementation `#frontend`
- [ ] Testing `#testing`

### 15. Export/Import Settings
- [ ] Backend implementation `#backend`
- [ ] Frontend implementation `#frontend`
- [ ] Testing `#testing`

---

## Notes

- Strike through completed tasks with `~~text~~` but keep them in the list
- Move active tasks to `active-tracker.md`
- Update this file after completing each task
- Add new tasks as they are discovered
