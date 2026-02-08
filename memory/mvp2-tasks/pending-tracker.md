# MVP2 Pending Tasks Tracker

**Last Updated**: 2026-02-06  
**Status**: Phase 0 - Prerequisites

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
- [ ] Add MVP2 dependencies `#backend`
  - [ ] Add rusqlite for SQLite
  - [ ] Add serde/serde_json for serialization
  - [ ] Add chrono for timestamps
  - [ ] Add uuid for unique IDs
  - [ ] Add tracing/tracing-subscriber for logging
  - [ ] Add arboard for clipboard
  - [ ] Add notify-rust for notifications
  - [ ] Add thiserror for error handling
  - [ ] Run `cargo audit` for security check

### Frontend Refactoring
- [ ] Restructure frontend `#critical` `#frontend`
  - [ ] Create component library (`common/`, `download/`, `queue/`, `history/`, `settings/`)
  - [ ] Set up Zustand for state management
  - [ ] Create stores (downloadStore, queueStore, settingsStore, historyStore)
  - [ ] Create custom hooks (useDownload, useQueue, useSettings)
  - [ ] Create utility functions (formatBytes, formatSpeed, formatTime)
  - [ ] Convert all components to TypeScript

### Error Handling
- [/] Implement error handling strategy `#backend` **ACTIVE**
  - [ ] Create `core/error.rs` with custom error types
  - [ ] Add thiserror dependency
  - [ ] Define DownloadError enum
  - [ ] Update all modules to use Result<T, DownloadError>
  - [ ] Add error context to all error paths

### Documentation
- [ ] Document codebase `#docs`
  - [ ] Add rustdoc comments to all public APIs
  - [ ] Create CONTRIBUTING.md
  - [ ] Update README.md with new structure
  - [ ] Document module architecture

---

## Phase 1: Essential Features (Priority 1)

### 1. Pause/Resume Downloads
- [ ] Backend implementation `#backend`
  - [ ] Design pause state structure
  - [ ] Save download state to JSON
  - [ ] Implement pause command
  - [ ] Implement resume command
  - [ ] Load state on app restart
- [ ] Frontend implementation `#frontend`
  - [ ] Add Pause/Resume buttons to UI
  - [ ] Update progress bar on resume
  - [ ] Show paused state visually
- [ ] Testing `#testing`
  - [ ] Test pause mid-download
  - [ ] Test resume after app restart
  - [ ] Test no data loss on pause/resume

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
