# Pirate Downloader - MVP2 Product Requirements Document

## Executive Summary

Transform Pirate Downloader from a basic multi-threaded download tool into a **full-featured, production-ready download manager** that rivals IDM, Free Download Manager, and other industry leaders.

**Current State (MVP1)**: ✅ Rock-solid download engine with 100% completion rate  
**Target State (MVP2)**: 🎯 Complete download management solution with advanced features

---

## Core Philosophy

1. **Reliability First** - Never compromise the 100% completion guarantee
2. **User Experience** - Intuitive, beautiful, and fast
3. **Power User Features** - Advanced controls for those who need them
4. **Cross-Platform** - Windows, macOS, Linux support

---

## Prerequisites (Phase 0: Foundation)

Before starting MVP2 development, we must establish a solid foundation to ensure maintainability, testability, and scalability.

### 🏗️ 1. Codebase Refactoring (CRITICAL)

**Problem**: Currently, all download logic lives in `lib.rs` (~333 lines). Adding MVP2 features will balloon this to 1000+ lines, making it unmaintainable.

**Solution**: Modular architecture with clear separation of concerns.

#### Proposed File Structure

```
src-tauri/
├── src/
│   ├── main.rs                    # Entry point (Tauri setup only)
│   ├── lib.rs                     # Public API (Tauri commands only)
│   │
│   ├── core/                      # Core download engine
│   │   ├── mod.rs                 # Module exports
│   │   ├── downloader.rs          # Main download orchestrator
│   │   ├── chunk.rs               # Chunk management & retry logic
│   │   ├── worker.rs              # Worker thread implementation
│   │   ├── integrity.rs           # Verification & byte counting
│   │   └── types.rs               # Shared types (DownloadState, ChunkInfo, etc.)
│   │
│   ├── storage/                   # Persistence layer
│   │   ├── mod.rs
│   │   ├── database.rs            # SQLite connection & migrations
│   │   ├── downloads.rs           # Downloads table CRUD
│   │   ├── history.rs             # History table CRUD
│   │   ├── settings.rs            # Settings table CRUD
│   │   └── categories.rs          # Categories table CRUD
│   │
│   ├── queue/                     # Queue management
│   │   ├── mod.rs
│   │   ├── manager.rs             # Queue orchestrator (add, remove, reorder)
│   │   ├── state.rs               # Queue state machine (pending → active → complete)
│   │   └── persistence.rs         # Save/load queue to disk
│   │
│   ├── network/                   # Network utilities
│   │   ├── mod.rs
│   │   ├── client.rs              # HTTP client configuration
│   │   ├── headers.rs             # Header parsing (filename, size, etc.)
│   │   └── speed_limiter.rs       # Bandwidth limiting (token bucket)
│   │
│   ├── utils/                     # Utilities
│   │   ├── mod.rs
│   │   ├── filesystem.rs          # File operations (allocation, verification)
│   │   ├── sanitize.rs            # Filename sanitization
│   │   └── logger.rs              # Structured logging
│   │
│   └── integrations/              # External integrations (Phase 2+)
│       ├── mod.rs
│       ├── clipboard.rs           # Clipboard monitoring
│       ├── notifications.rs       # Desktop notifications
│       └── tray.rs                # System tray
│
├── Cargo.toml
└── tauri.conf.json
```

#### Module Responsibilities

| Module | Responsibility | Max Lines |
|--------|---------------|-----------|
| `lib.rs` | Tauri command definitions only | ~100 |
| `core/downloader.rs` | Orchestrate download lifecycle | ~200 |
| `core/chunk.rs` | Chunk logic, retry tracking | ~150 |
| `core/worker.rs` | Worker thread implementation | ~200 |
| `core/integrity.rs` | Byte verification, checksums | ~100 |
| `storage/database.rs` | SQLite setup, migrations | ~150 |
| `storage/downloads.rs` | Downloads CRUD | ~200 |
| `queue/manager.rs` | Queue operations | ~200 |
| `network/client.rs` | HTTP client setup | ~100 |

**Acceptance Criteria**:
- ✅ No single file exceeds 300 lines
- ✅ Each module has a single, clear responsibility
- ✅ All modules have unit tests
- ✅ Public APIs are documented with rustdoc

---

### 🧪 2. Testing Framework

**Problem**: Currently no automated tests. Adding features without tests will cause regressions.

**Solution**: Comprehensive test coverage before MVP2.

#### Test Structure

```
src-tauri/
├── src/
│   └── (modules as above)
├── tests/
│   ├── integration/
│   │   ├── download_flow.rs      # End-to-end download tests
│   │   ├── pause_resume.rs       # Pause/resume scenarios
│   │   └── queue_management.rs   # Queue operations
│   └── fixtures/
│       ├── test_server.rs         # Mock HTTP server
│       └── test_files.rs          # Test file generators
└── Cargo.toml
```

#### Test Coverage Goals

- **Unit Tests**: 80% coverage for core modules
- **Integration Tests**: All critical user flows
- **Mock Server**: Simulate slow/failing connections
- **Benchmarks**: Performance regression tests

**Tools**:
- `cargo test` - Standard Rust testing
- `mockito` - HTTP mocking
- `criterion` - Benchmarking
- `cargo-tarpaulin` - Coverage reports

**Acceptance Criteria**:
- ✅ All core modules have unit tests
- ✅ Integration tests for download, pause, resume
- ✅ CI runs tests on every commit
- ✅ Coverage reports generated automatically

---

### 📊 3. Logging & Observability

**Problem**: Current logging is ad-hoc `println!` statements. Debugging production issues is difficult.

**Solution**: Structured logging with levels and filtering.

#### Implementation

```rust
// Use `tracing` crate for structured logging
use tracing::{info, warn, error, debug, instrument};

#[instrument(skip(url))]
async fn download_chunk(chunk_id: u64, url: &str) -> Result<Vec<u8>> {
    debug!(chunk_id, "Starting chunk download");
    
    match fetch_chunk(url).await {
        Ok(data) => {
            info!(chunk_id, bytes = data.len(), "Chunk downloaded successfully");
            Ok(data)
        }
        Err(e) => {
            error!(chunk_id, error = %e, "Chunk download failed");
            Err(e)
        }
    }
}
```

**Features**:
- Log levels: TRACE, DEBUG, INFO, WARN, ERROR
- Structured fields (chunk_id, bytes, speed, etc.)
- File rotation (max 10MB per file, keep 5 files)
- User-configurable verbosity in settings

**Acceptance Criteria**:
- ✅ All modules use `tracing` instead of `println!`
- ✅ Logs saved to `~/.pirate-downloader/logs/`
- ✅ Settings UI has "Debug Mode" toggle
- ✅ Logs include timestamps, thread IDs, and context

---

### 🔄 4. CI/CD Pipeline

**Problem**: Manual testing is error-prone. Need automated checks on every commit.

**Solution**: GitHub Actions workflow for testing, linting, and building.

#### Workflow

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
      - run: cargo fmt --check

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: cargo tarpaulin --out Xml
      - uses: codecov/codecov-action@v3
```

**Checks**:
- ✅ Tests pass on Windows, macOS, Linux
- ✅ No clippy warnings
- ✅ Code formatted with `rustfmt`
- ✅ Coverage reports uploaded to Codecov

---

### 📦 5. Dependency Management

**Problem**: Need to add new dependencies for MVP2 (SQLite, notifications, etc.). Must ensure compatibility.

**Solution**: Carefully vetted dependency list with version pinning.

#### New Dependencies

```toml
[dependencies]
# Existing
reqwest = { version = "0.11", features = ["stream"] }
tokio = { version = "1.35", features = ["full"] }
tauri = { version = "2.0", features = ["dialog", "notification"] }

# New for MVP2
rusqlite = { version = "0.30", features = ["bundled"] }  # SQLite
serde = { version = "1.0", features = ["derive"] }       # Serialization
serde_json = "1.0"                                        # JSON
chrono = "0.4"                                            # Timestamps
uuid = { version = "1.6", features = ["v4"] }            # Unique IDs
tracing = "0.1"                                           # Logging
tracing-subscriber = "0.3"                                # Log output
arboard = "3.3"                                           # Clipboard
notify-rust = "4.10"                                      # Notifications

[dev-dependencies]
mockito = "1.2"                                           # HTTP mocking
criterion = "0.5"                                         # Benchmarking
tempfile = "3.8"                                          # Temp files for tests
```

**Acceptance Criteria**:
- ✅ All dependencies have stable versions (no 0.x)
- ✅ Security audit passes (`cargo audit`)
- ✅ No duplicate dependencies
- ✅ Bundle size remains < 10MB

---

### 🎨 6. Frontend Refactoring

**Problem**: Current UI is minimal. Need component library for MVP2 features.

**Solution**: Component-based architecture with state management.

#### Frontend Structure

```
src/
├── components/
│   ├── common/                    # Reusable components
│   │   ├── Button.tsx
│   │   ├── Input.tsx
│   │   ├── ProgressBar.tsx
│   │   └── Modal.tsx
│   ├── download/                  # Download-specific
│   │   ├── DownloadItem.tsx       # Single download card
│   │   ├── DownloadList.tsx       # List of downloads
│   │   └── AddDownloadDialog.tsx  # Add URL modal
│   ├── queue/
│   │   ├── QueueView.tsx          # Queue management
│   │   └── QueueItem.tsx          # Single queue item
│   ├── history/
│   │   ├── HistoryView.tsx        # History list
│   │   └── HistoryItem.tsx        # Single history entry
│   └── settings/
│       ├── SettingsPanel.tsx      # Main settings
│       └── SettingSection.tsx     # Settings group
├── stores/
│   ├── downloadStore.ts           # Download state (Zustand)
│   ├── queueStore.ts              # Queue state
│   ├── settingsStore.ts           # Settings state
│   └── historyStore.ts            # History state
├── hooks/
│   ├── useDownload.ts             # Download operations
│   ├── useQueue.ts                # Queue operations
│   └── useSettings.ts             # Settings operations
├── utils/
│   ├── formatBytes.ts             # Size formatting
│   ├── formatSpeed.ts             # Speed formatting
│   └── formatTime.ts              # Duration formatting
└── App.tsx
```

**State Management**: Zustand (lightweight, TypeScript-friendly)

**Acceptance Criteria**:
- ✅ All components are TypeScript
- ✅ Reusable components in `common/`
- ✅ State management with Zustand
- ✅ No prop drilling (use stores)

---

### 🔐 7. Error Handling Strategy

**Problem**: Current error handling is inconsistent. Need unified approach.

**Solution**: Custom error types with context.

#### Error Types

```rust
// src/core/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Chunk {chunk_id} failed after {attempts} attempts")]
    ChunkFailed { chunk_id: u64, attempts: u32 },
    
    #[error("Integrity check failed: {downloaded} / {expected} bytes")]
    IntegrityFailed { downloaded: u64, expected: u64 },
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

pub type Result<T> = std::result::Result<T, DownloadError>;
```

**Acceptance Criteria**:
- ✅ All errors use custom types
- ✅ Errors include context (chunk_id, URL, etc.)
- ✅ User-friendly error messages in UI
- ✅ Errors logged with full context

---

## Summary: Phase 0 Checklist

Before starting MVP2 feature development:

- [ ] **Refactor codebase** into modular structure
- [ ] **Add unit tests** for all core modules
- [ ] **Set up CI/CD** pipeline (GitHub Actions)
- [ ] **Implement structured logging** (tracing)
- [ ] **Add dependencies** for MVP2 (SQLite, etc.)
- [ ] **Refactor frontend** with component library
- [ ] **Implement error handling** strategy
- [ ] **Documentation** (rustdoc for all public APIs)

**Estimated Time**: 1-2 weeks  
**Priority**: CRITICAL (blocks all MVP2 work)

---

## Feature Categories

### 🎯 Priority 1: Essential Features (Must Have)

#### 1. **Download Control: Pause/Resume/Stop/Cancel**
**Problem**: Users need flexible control over downloads with different stopping behaviors  
**Solution**: Four distinct control operations with clear state management

**Control Operations**:

1. **Pause** 🟡
   - **Behavior**: Temporarily halt download, preserve all state
   - **State**: Download remains in queue as "paused"
   - **Data**: All progress saved, chunks preserved
   - **Resume**: Continue from exact byte position
   - **Use Case**: Temporarily free bandwidth, continue later

2. **Resume** ▶️
   - **Behavior**: Continue paused download from last position
   - **State**: Change from "paused" to "active"
   - **Data**: Load saved state, rebuild incomplete chunk queue
   - **Use Case**: Resume after pause or app restart

3. **Stop** ⏹️
   - **Behavior**: Gracefully stop download, save final state
   - **State**: Move to "stopped" (can resume later)
   - **Data**: Save all progress, keep partial file
   - **File**: Partial file remains on disk with `.part` extension
   - **Use Case**: Stop for now, may resume in future

4. **Cancel** ❌
   - **Behavior**: Immediately terminate download, cleanup everything
   - **State**: Remove from queue entirely (works from **any state**: active, paused, stopped, failed)
   - **Data**: Delete all state files and partial downloads
   - **File**: Delete `.part` file from disk
   - **Use Case**: Don't want this file anymore, free up space
   - **Note**: Available as an option for all downloads regardless of current state

**Technical Requirements**:
- **State Persistence**:
  - Save completed chunk list to JSON file
  - Store metadata (URL, filepath, total size, completed bytes, thread count)
  - Track download status (active, paused, stopped, completed, failed, cancelled)
  - Timestamp for pause/stop/resume events

- **Resume Logic**:
  - Load saved state from disk
  - Verify partial file exists and matches expected size
  - Rebuild chunk queue with only incomplete chunks
  - Resume with same thread count as original download

- **File Management**:
  - Paused/Stopped: Keep `.part` file
  - Cancelled: Delete `.part` file and state JSON
  - Completed: Rename `.part` to final filename

- **UI Controls**:
  - **Active Download**: Show Pause, Stop, Cancel buttons
  - **Paused Download**: Show Resume, Cancel buttons
  - **Stopped Download**: Show Resume, Cancel buttons
  - **Failed Download**: Show Retry, Cancel buttons
  - **Cancel button**: Always available regardless of state
  - Confirmation dialog for Cancel (destructive action)

**State Transitions**:
```
                    ┌─────────────┐
                    │   CANCEL    │ ← Can cancel from ANY state
                    └─────────────┘
                           ↑
        ┌──────────────────┼──────────────────┐
        │                  │                  │
pending → active → paused ─┘    completed     │
              ↓       ↓                       │
            stopped  failed ───────────────────┘
              ↓
            resumed → active
```

**Acceptance Criteria**:
- ✅ Can pause mid-download without data loss
- ✅ Can resume paused download from exact position
- ✅ Can stop download and resume later (even after app restart)
- ✅ Can cancel download with full cleanup (file + state deleted)
- ✅ No re-downloading of completed chunks on resume
- ✅ Progress bar shows correct percentage on resume
- ✅ Confirmation dialog before cancel
- ✅ Stopped downloads persist across app restarts
- ✅ UI clearly shows current state (active/paused/stopped)

---

#### 2. **Download Queue Management**
**Problem**: Users want to download multiple files sequentially or in parallel  
**Solution**: Queue system with configurable concurrency

**Technical Requirements**:
- Queue data structure (pending, active, completed, failed)
- Configurable max concurrent downloads (1-5)
- Drag-to-reorder queue
- Auto-start next download when one completes
- Persistent queue (survives app restart)

**UI Components**:
- Queue list with status badges
- Move up/down buttons
- Start/Stop/Remove buttons per item
- Global "Start All" / "Pause All" buttons

**Acceptance Criteria**:
- ✅ Can add multiple URLs to queue
- ✅ Downloads start automatically based on concurrency limit
- ✅ Can reorder queue
- ✅ Queue persists across app restarts

---

#### 3. **Automatic Filename Detection**
**Problem**: Users shouldn't have to manually name files  
**Solution**: Extract filename from Content-Disposition header or URL

**Technical Requirements**:
- Parse `Content-Disposition: attachment; filename="file.zip"`
- Fallback to URL path extraction
- Sanitize filename (remove invalid characters)
- Handle duplicate filenames (append `(1)`, `(2)`, etc.)
- Allow user override before download starts

**Acceptance Criteria**:
- ✅ Automatically detects filename from server
- ✅ Shows preview before download
- ✅ Handles duplicates gracefully
- ✅ User can edit filename

---

#### 4. **Download History**
**Problem**: Users need to track what they've downloaded  
**Solution**: Persistent history database with search

**Technical Requirements**:
- SQLite database for history
- Store: URL, filename, size, date, duration, avg speed
- Search by filename or URL
- Filter by date range
- "Open file location" button
- "Re-download" button
- Clear history option

**Acceptance Criteria**:
- ✅ All downloads saved to history
- ✅ Can search history
- ✅ Can re-download from history
- ✅ Can clear history

---

#### 5. **Settings & Configuration**
**Problem**: Users need to customize behavior  
**Solution**: Comprehensive settings panel

**Settings Categories**:

**General**:
- Default download directory
- Max concurrent downloads (1-5)
- Default thread count (1-64)
- Auto-start downloads on add
- Close to system tray

**Network**:
- Bandwidth limit (KB/s, MB/s, or unlimited)
- Timeout settings (connect, read)
- Retry attempts per chunk
- Speed enforcement threshold

**UI**:
- Theme (Light/Dark/System)
- Language (English, Spanish, etc.)
- Notification preferences
- Sound on completion

**Advanced**:
- Chunk size strategy (auto, manual)
- Enable/disable speed enforcer
- Debug logging

**Acceptance Criteria**:
- ✅ Settings persist across restarts
- ✅ Changes apply immediately
- ✅ Validation for invalid inputs

---

### 🚀 Priority 2: Power User Features (Should Have)

#### 6. **Bandwidth Limiter**
**Problem**: Users don't want downloads to saturate their connection  
**Solution**: Global and per-download speed limits

**Technical Requirements**:
- Token bucket algorithm for rate limiting
- Global limit (affects all downloads)
- Per-download limit (overrides global)
- UI: Slider or input field (KB/s)
- Real-time adjustment (no restart needed)

**Acceptance Criteria**:
- ✅ Can set global speed limit
- ✅ Can set per-download limit
- ✅ Limits are enforced accurately (±5%)
- ✅ Can disable limits

---

#### 7. **Browser Integration**
**Problem**: Users want to capture downloads from their browser  
**Solution**: Browser extension + native messaging

**Technical Requirements**:
- Chrome/Firefox extension
- Intercept download requests
- Send to Pirate Downloader via native messaging
- Auto-categorize by file type
- Option to disable for small files (<1MB)

**Acceptance Criteria**:
- ✅ Extension captures downloads
- ✅ Sends to app seamlessly
- ✅ User can enable/disable per site
- ✅ Works on Chrome and Firefox

---

#### 8. **Download Scheduling**
**Problem**: Users want to download during off-peak hours  
**Solution**: Schedule downloads for specific times

**Technical Requirements**:
- Set start time for queued downloads
- "Download between X and Y" time windows
- Pause downloads outside time window
- Calendar UI for scheduling

**Acceptance Criteria**:
- ✅ Can schedule download for future time
- ✅ Downloads start automatically at scheduled time
- ✅ Can set recurring schedules
- ✅ Notifications when scheduled download starts

---

#### 9. **Categories & Organization**
**Problem**: Users download many files and lose track  
**Solution**: Auto-categorization and manual folders

**Technical Requirements**:
- Auto-categorize by file type:
  - Videos (.mp4, .mkv, .avi)
  - Music (.mp3, .flac, .wav)
  - Documents (.pdf, .docx, .xlsx)
  - Archives (.zip, .rar, .7z)
  - Software (.exe, .dmg, .deb)
  - Other
- Custom categories
- Separate download folders per category
- Filter view by category

**Acceptance Criteria**:
- ✅ Files auto-categorized correctly
- ✅ Can create custom categories
- ✅ Can move files between categories
- ✅ Each category has separate folder

---

#### 10. **Clipboard Monitoring**
**Problem**: Users copy URLs and want instant download  
**Solution**: Monitor clipboard for URLs

**Technical Requirements**:
- Detect URLs in clipboard
- Show popup: "Download this URL?"
- Configurable (on/off, whitelist domains)
- Regex patterns for supported sites

**Acceptance Criteria**:
- ✅ Detects URLs automatically
- ✅ Shows non-intrusive prompt
- ✅ Can whitelist/blacklist domains
- ✅ Can disable feature

---

### 🎨 Priority 3: UX Enhancements (Nice to Have)

#### 11. **System Tray Integration**
**Problem**: Users want app to run in background  
**Solution**: Minimize to system tray

**Technical Requirements**:
- Tray icon with context menu
- Show active downloads count
- Quick actions: Pause All, Resume All, Exit
- Click to restore window
- Notifications from tray

---

#### 12. **Desktop Notifications**
**Problem**: Users want to know when downloads complete  
**Solution**: Native OS notifications

**Technical Requirements**:
- Notify on download complete
- Notify on download failed
- Notify on all downloads complete
- Click notification to open file/folder
- Configurable (on/off, sound)

---

#### 13. **Drag & Drop Support**
**Problem**: Users want easy URL/file addition  
**Solution**: Drag URLs or torrent files into app

**Technical Requirements**:
- Drag URL from browser → starts download
- Drag .torrent file → adds to queue
- Drag multiple URLs → batch add
- Visual feedback on drag-over

---

#### 14. **Download Verification**
**Problem**: Users want to verify file integrity  
**Solution**: Checksum verification (MD5, SHA256)

**Technical Requirements**:
- Calculate checksum after download
- Compare with user-provided hash
- Show verification status in UI
- Support MD5, SHA1, SHA256

---

#### 15. **Export/Import Settings**
**Problem**: Users want to transfer settings between machines  
**Solution**: Export settings to JSON

**Technical Requirements**:
- Export all settings to JSON file
- Import settings from JSON
- Include queue state (optional)
- Backup/restore functionality

---

### 🔧 Priority 4: Advanced Features (Future)

#### 16. **Torrent Support**
- BitTorrent protocol integration
- Magnet link support
- Peer management
- Seeding after download

#### 17. **Video Streaming**
- Stream while downloading
- Built-in video player
- Subtitle support

#### 18. **Cloud Storage Integration**
- Upload to Google Drive, Dropbox, OneDrive
- Auto-upload on completion
- Sync across devices

#### 19. **Download Analytics**
- Total data downloaded
- Average speeds over time
- Most downloaded file types
- Charts and graphs

#### 20. **Proxy Support**
- HTTP/HTTPS/SOCKS5 proxy
- Per-download proxy settings
- Proxy authentication

---

## Technical Architecture

### Database Schema (SQLite)

```sql
-- Downloads table
CREATE TABLE downloads (
    id INTEGER PRIMARY KEY,
    url TEXT NOT NULL,
    filename TEXT NOT NULL,
    filepath TEXT NOT NULL,
    total_size INTEGER,
    downloaded_bytes INTEGER,
    status TEXT, -- pending, active, paused, completed, failed
    category TEXT,
    created_at TIMESTAMP,
    completed_at TIMESTAMP,
    avg_speed REAL,
    metadata JSON -- chunks, retry counts, etc.
);

-- History table
CREATE TABLE history (
    id INTEGER PRIMARY KEY,
    url TEXT,
    filename TEXT,
    size INTEGER,
    downloaded_at TIMESTAMP,
    duration REAL,
    avg_speed REAL
);

-- Settings table
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT
);

-- Categories table
CREATE TABLE categories (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE,
    download_path TEXT,
    file_extensions TEXT -- JSON array
);
```

### File Structure

```
src-tauri/
├── src/
│   ├── lib.rs              (existing download engine)
│   ├── database.rs         (SQLite operations)
│   ├── queue.rs            (queue management)
│   ├── settings.rs         (settings CRUD)
│   ├── clipboard.rs        (clipboard monitoring)
│   ├── notifications.rs    (desktop notifications)
│   └── utils.rs            (helpers)
src/
├── components/
│   ├── DownloadQueue.tsx   (queue UI)
│   ├── DownloadItem.tsx    (single download)
│   ├── Settings.tsx        (settings panel)
│   ├── History.tsx         (history view)
│   └── AddDownload.tsx     (add URL dialog)
├── stores/
│   ├── downloadStore.ts    (Zustand/Redux)
│   └── settingsStore.ts
└── App.tsx
```

---

## UI/UX Design Principles

### Layout
- **Sidebar**: Queue, History, Settings, Categories
- **Main Area**: Active downloads with progress bars
- **Top Bar**: Add URL, global controls, search
- **Bottom Bar**: Total speed, active downloads count

### Color Scheme
- **Light Mode**: Clean whites, subtle grays, accent blue
- **Dark Mode**: Deep grays, vibrant accents, high contrast

### Animations
- Smooth progress bar updates (60fps)
- Fade in/out for notifications
- Slide transitions for panel switches

---

## Development Phases

### Phase 1: Foundation (Week 1-2)
- ✅ Database setup (SQLite)
- ✅ Settings system
- ✅ Queue management (backend)
- ✅ Pause/Resume functionality

### Phase 2: Core Features (Week 3-4)
- ✅ Download history
- ✅ Automatic filename detection
- ✅ Categories & organization
- ✅ UI redesign for queue

### Phase 3: Power Features (Week 5-6)
- ✅ Bandwidth limiter
- ✅ Clipboard monitoring
- ✅ System tray integration
- ✅ Notifications

### Phase 4: Advanced (Week 7-8)
- ✅ Browser extension
- ✅ Download scheduling
- ✅ Drag & drop
- ✅ Checksum verification

### Phase 5: Polish (Week 9-10)
- ✅ Testing & bug fixes
- ✅ Performance optimization
- ✅ Documentation
- ✅ Production build

---

## Success Metrics

### Performance
- Download speed: 20+ MB/s average
- 100% completion rate maintained
- <100ms UI response time
- <50MB RAM usage (idle)

### User Experience
- <3 clicks to start download
- <5 seconds to add URL
- Intuitive UI (no manual needed)
- Zero crashes in 1000 downloads

### Feature Adoption
- 80% users use pause/resume
- 60% users use queue
- 40% users customize settings
- 30% users use browser extension

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Database corruption | High | Regular backups, WAL mode |
| Browser extension approval | Medium | Follow store guidelines strictly |
| Performance degradation | High | Profiling, benchmarking |
| Cross-platform bugs | Medium | Test on all OSes |

---

## Competitive Analysis

| Feature | Pirate DL | IDM | FDM | JDownloader |
|---------|-----------|-----|-----|-------------|
| Multi-threading | ✅ 32 | ✅ 32 | ✅ 16 | ✅ 20 |
| Pause/Resume | 🔜 | ✅ | ✅ | ✅ |
| Browser Integration | 🔜 | ✅ | ✅ | ✅ |
| Open Source | ✅ | ❌ | ❌ | ✅ |
| Cross-Platform | ✅ | ❌ | ✅ | ✅ |
| Modern UI | ✅ | ❌ | ⚠️ | ❌ |
| 100% Completion | ✅ | ⚠️ | ⚠️ | ✅ |

**Our Advantage**: Modern UI + Open Source + 100% reliability

---

## Next Steps

1. **Review & Approve** this PRD
2. **Prioritize** features (which to build first?)
3. **Create** implementation plan for Phase 1
4. **Design** UI mockups
5. **Start** development!

---

## Questions for Discussion

1. Should we support torrents in MVP2 or defer to MVP3?
2. Which browser(s) to support first? (Chrome, Firefox, both?)
3. Freemium model or 100% free forever?
4. Cloud storage integration priority?
5. Mobile app (iOS/Android) in roadmap?

---

**Status**: 📋 Draft - Awaiting Approval  
**Version**: 1.0  
**Last Updated**: 2026-02-05  
**Author**: Antigravity AI + User
