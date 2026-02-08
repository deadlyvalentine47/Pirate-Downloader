# Logging & Error Handling Implementation Plan

**Date**: 2026-02-07  
**Task**: Implement Structured Logging & Error Handling  
**Status**: Planning Phase

---

## Current State Analysis

### println! Usage
**Total**: 18 println! statements across codebase

**lib.rs** (13 statements):
- Line 39: Download start message
- Line 40-82: File size and thread info
- Line 136: Chunk retry attempts
- Line 199-211: Chunk completion messages
- Line 226: Chunk failure messages
- Line 243-250: Download summary statistics
- Line 275: HEAD request fallback message

**core/integrity.rs** (5 statements):
- Line 26-28: Integrity verification messages
- Line 42: Integrity check passed message

### Error Handling Patterns
**Current approach**: Using `String` for errors
- `Result<T, String>` return types
- `.map_err(|e| e.to_string())` for error conversion
- `.map_err(|e| format!("...", e))` for custom messages

**Unwrap usage**: 4 instances (potential panic points)
- Line 103: File writer creation
- Line 123: Queue pop
- Line 234: Writer flush
- Line 274: Response status check

---

## Implementation Strategy

### Phase 1: Add Dependencies
Add to `Cargo.toml`:
```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "json"] }
tracing-appender = "0.2"
thiserror = "1.0"
```

### Phase 2: Create Error Types
Create `src/core/error.rs` with:
- `DownloadError` enum covering all error cases
- Implement `From` traits for common error types
- Use `thiserror` for automatic Display/Error implementations

Error categories:
1. **NetworkError** - HTTP request failures, connection issues
2. **FileSystemError** - File I/O, allocation, permission issues
3. **IntegrityError** - Incomplete downloads, verification failures
4. **ParseError** - Header parsing, URL parsing failures
5. **ConfigError** - Invalid settings, configuration issues

### Phase 3: Create Logger Module
Create `src/utils/logger.rs` with:
- Logger initialization function
- File appender with rotation (daily, 10MB max)
- Console output for development
- Configurable log levels (TRACE, DEBUG, INFO, WARN, ERROR)
- Structured logging with context fields

### Phase 4: Replace println! with Tracing
**Mapping strategy**:
- Download start/end → `info!`
- Chunk progress → `debug!`
- Retry attempts → `warn!`
- Failures → `error!`
- Summary stats → `info!`
- Verification → `info!`

**Context fields to add**:
- `download_id` - Unique identifier for each download
- `url` - Download URL
- `chunk_id` - Chunk index
- `retry_count` - Number of retries
- `bytes_downloaded` - Progress tracking

### Phase 5: Update Error Handling
**Changes needed**:
1. Update all function signatures to return `Result<T, DownloadError>`
2. Replace `.unwrap()` with proper error handling
3. Add error context using `map_err` with custom messages
4. Propagate errors up the call stack
5. Log errors at appropriate levels before returning

**Priority unwraps to fix**:
1. Line 103: File writer - return FileSystemError
2. Line 123: Queue pop - handle gracefully (shouldn't fail)
3. Line 234: Writer flush - return FileSystemError
4. Line 274: Response check - return NetworkError

---

## Implementation Order

### Step 1: Dependencies & Error Types (Day 1 Morning)
- [ ] Add dependencies to Cargo.toml
- [ ] Create `core/error.rs` with DownloadError enum
- [ ] Implement From traits for common errors
- [ ] Test error type compilation

### Step 2: Logger Setup (Day 1 Afternoon)
- [ ] Create `utils/logger.rs`
- [ ] Implement logger initialization
- [ ] Set up file rotation
- [ ] Add log level configuration
- [ ] Test logger writes to file

### Step 3: Update lib.rs (Day 2 Morning)
- [ ] Replace println! with tracing macros
- [ ] Update function signatures to use DownloadError
- [ ] Fix unwrap() calls
- [ ] Add error context
- [ ] Test download still works

### Step 4: Update Modules (Day 2 Afternoon)
- [ ] Update core/integrity.rs logging
- [ ] Update network/ modules error handling
- [ ] Update utils/ modules error handling
- [ ] Ensure all modules use DownloadError

### Step 5: Testing & Verification (Day 3)
- [ ] Test logging in different scenarios
- [ ] Verify log files are created
- [ ] Test log rotation
- [ ] Verify error messages are helpful
- [ ] Test error propagation
- [ ] Performance test (ensure no regression)

---

## Log File Structure

**Location**: `%APPDATA%/PirateDownloader/logs/`  
**Naming**: `pirate-downloader-YYYY-MM-DD.log`  
**Rotation**: Daily or 10MB max size  
**Retention**: Keep last 7 days

**Format**:
```
2026-02-07T12:30:45.123Z INFO  download_file{url="https://..." download_id="abc123"}: Starting download
2026-02-07T12:30:45.456Z DEBUG chunk_worker{chunk_id=5 download_id="abc123"}: Chunk complete bytes=16777216
2026-02-07T12:30:46.789Z WARN  chunk_worker{chunk_id=12 download_id="abc123"}: Retry attempt retry_count=2
2026-02-07T12:30:50.123Z INFO  download_file{download_id="abc123"}: Download complete duration=4.567s speed=21.5MB/s
```

---

## Error Type Design

```rust
#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    
    #[error("Integrity check failed: {message}")]
    Integrity { message: String },
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
}
```

---

## Testing Strategy

### Unit Tests
- Test error type conversions
- Test logger initialization
- Test log level filtering

### Integration Tests
- Download with logging enabled
- Verify log file creation
- Test error scenarios (network failure, disk full)
- Verify error messages are logged

### Performance Tests
- Benchmark logging overhead
- Ensure < 5% performance impact
- Test with high-frequency logging (chunk progress)

---

## Success Criteria

✅ All println! statements replaced with tracing  
✅ Custom error types implemented  
✅ All unwrap() calls removed  
✅ Log files created and rotated  
✅ Error messages are helpful and actionable  
✅ No performance regression (< 5% overhead)  
✅ All tests passing  
✅ Documentation updated

---

## Risks & Mitigation

**Risk**: Logging overhead impacts performance  
**Mitigation**: Use async logging, buffer writes, test thoroughly

**Risk**: Breaking existing functionality  
**Mitigation**: Incremental changes, test after each module

**Risk**: Log files fill disk space  
**Mitigation**: Implement rotation and retention policies

**Risk**: Error types too complex  
**Mitigation**: Start simple, add variants as needed

---

## Notes

- Keep logging concise - avoid logging in tight loops
- Use structured fields instead of string formatting
- Log errors at the point they occur, not just at boundaries
- Consider adding download_id to all log entries for traceability
- Use span! macro for request-scoped context
