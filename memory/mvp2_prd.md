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

## Feature Categories

### 🎯 Priority 1: Essential Features (Must Have)

#### 1. **Pause/Resume Downloads**
**Problem**: Users need to stop downloads and continue later without losing progress  
**Solution**: Save download state to disk, resume from exact byte position

**Technical Requirements**:
- Save completed chunk list to JSON file
- Store metadata (URL, filepath, total size, completed bytes)
- Resume by rebuilding queue with only incomplete chunks
- UI: Pause/Resume buttons per download

**Acceptance Criteria**:
- ✅ Can pause mid-download
- ✅ Can resume after app restart
- ✅ No data loss or re-downloading completed chunks
- ✅ Progress bar shows correct percentage on resume

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
