# Codebase Refactoring Implementation Plan

**Date**: 2026-02-06  
**Goal**: Refactor `lib.rs` into modular structure while maintaining 100% completion guarantee and performance  
**Current State**: Monolithic `lib.rs` with 333 lines  
**Target State**: Modular structure with clear separation of concerns  

---

## Safety-First Approach

### Critical Constraints
1. **NEVER** change the core download loop logic (lines 107-237)
2. **NEVER** change retry tracking or byte counting logic
3. **NEVER** change the integrity verification
4. **Test after EVERY extraction** - no exceptions
5. **Commit after each successful extraction**

### Testing Protocol
After each module extraction:
1. Build: `cargo build`
2. Run dev server: `npm run tauri dev`
3. Test download: 100+ MB file
4. Verify: 100% completion, correct byte count
5. Check speed: 18-24 MB/s average

---

## Phase 1: Extract Network Utilities (SAFEST)

### Module: `src/network/mod.rs` and `src/network/client.rs`

**What to extract**:
- HTTP client builder configuration
- User agent constant
- Timeout settings

**Current code** (lines 18-21, 95-100):
```rust
let client = reqwest::Client::builder()
    .user_agent("Mozilla/5.0...")
    .build()
```

**New module**:
```rust
// src/network/client.rs
pub fn create_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
}

pub fn create_worker_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .read_timeout(std::time::Duration::from_secs(5))
        .connect_timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap()
}
```

**Risk**: LOW - Simple function extraction  
**Test**: Verify download still works

---

## Phase 2: Extract Filename Parsing (SAFE)

### Module: `src/network/headers.rs`

**What to extract**:
- Content-Disposition header parsing
- URL-based filename extraction

**Current code** (lines 300-321 in `get_file_details`):
```rust
let mut filename = "download.dat".to_string();
if let Some(disp) = response.headers().get(CONTENT_DISPOSITION) {
    // ... parsing logic
}
```

**New module**:
```rust
// src/network/headers.rs
pub fn extract_filename(response: &reqwest::Response, url: &str) -> String {
    // Move all filename extraction logic here
}
```

**Risk**: LOW - Pure function, no side effects  
**Test**: Verify filename detection works

---

## Phase 3: Extract File System Utilities (SAFE)

### Module: `src/utils/filesystem.rs`

**What to extract**:
- Sparse file allocation
- Chunk size calculation

**Current code** (lines 38-54):
```rust
// Sparse file allocation
let mut file = File::create(&path)?;
file.seek(SeekFrom::Start(total_size - 1))?;
file.write_all(&[0])?;

// Dynamic chunking
let chunk_size = if total_size < 100 * 1024 * 1024 {
    512 * 1024
} else if ...
```

**New module**:
```rust
// src/utils/filesystem.rs
pub fn allocate_sparse_file(path: &Path, size: u64) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.seek(SeekFrom::Start(size - 1))?;
    file.write_all(&[0])?;
    Ok(())
}

pub fn calculate_chunk_size(total_size: u64) -> u64 {
    if total_size < 100 * 1024 * 1024 {
        512 * 1024
    } else if total_size < 1 * 1024 * 1024 * 1024 {
        4 * 1024 * 1024
    } else if total_size < 10 * 1024 * 1024 * 1024 {
        16 * 1024 * 1024
    } else {
        64 * 1024 * 1024
    }
}
```

**Risk**: LOW - Pure functions  
**Test**: Verify file allocation and chunking work

---

## Phase 4: Extract Core Types (SAFE)

### Module: `src/core/types.rs`

**What to extract**:
- Type aliases
- Constants
- Shared data structures

**New module**:
```rust
// src/core/types.rs
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use tokio::sync::Mutex;
use std::collections::{VecDeque, HashMap};

pub type ChunkQueue = Arc<Mutex<VecDeque<u64>>>;
pub type RetryTracker = Arc<Mutex<HashMap<u64, u32>>>;
pub type CompletionCounter = Arc<AtomicU64>;
pub type ByteCounter = Arc<AtomicU64>;
pub type SpeedStats = Arc<Mutex<(u64, f64, f64, std::time::Instant)>>;

pub const DEFAULT_THREADS: u64 = 8;
pub const CHUNK_RETRY_LIMIT: u32 = 5;
pub const SPEED_ENFORCEMENT_THRESHOLD: f64 = 300.0; // KB/s
pub const SPEED_ENFORCEMENT_DELAY: f64 = 3.0; // seconds
```

**Risk**: VERY LOW - Just type definitions  
**Test**: Verify compilation

---

## Phase 5: Extract Integrity Verification (MEDIUM RISK)

### Module: `src/core/integrity.rs`

**What to extract**:
- Final integrity check logic
- Byte verification

**Current code** (lines 257-279):
```rust
println!("Verifying integrity...");
let final_bytes = downloaded_bytes.load(Ordering::SeqCst);
// ... verification logic
```

**New module**:
```rust
// src/core/integrity.rs
pub fn verify_download(
    downloaded_bytes: u64,
    total_size: u64,
    completed_chunks: u64,
    total_chunks: u64,
) -> Result<(), String> {
    println!("Completed chunks: {} / {}", completed_chunks, total_chunks);
    println!(
        "Downloaded bytes: {} / {} ({:.2}%)",
        downloaded_bytes,
        total_size,
        (downloaded_bytes as f64 / total_size as f64) * 100.0
    );

    if downloaded_bytes < total_size {
        return Err(format!(
            "Download FAILED: {} / {} bytes ({} / {} chunks). Retry.",
            downloaded_bytes, total_size, completed_chunks, total_chunks
        ));
    }

    println!("Integrity Check PASSED: 100%");
    Ok(())
}
```

**Risk**: MEDIUM - Critical for reliability  
**Test**: Verify integrity check still catches incomplete downloads

---

## Phase 6: Create Module Structure

### Directory Structure
```
src-tauri/src/
├── lib.rs (Tauri commands only)
├── core/
│   ├── mod.rs
│   ├── types.rs
│   └── integrity.rs
├── network/
│   ├── mod.rs
│   ├── client.rs
│   └── headers.rs
└── utils/
    ├── mod.rs
    └── filesystem.rs
```

---

## Phase 7: Update lib.rs (FINAL STEP)

**After all modules are extracted and tested**:
- Import from modules
- Replace inline code with module calls
- Keep download loop logic EXACTLY as is
- Only change: use module functions instead of inline code

---

## What NOT to Extract (Keep in lib.rs)

### Core Download Loop (lines 107-237)
**REASON**: This is the heart of the 100% completion guarantee. Any changes risk breaking:
- Retry tracking
- Byte counting
- Speed enforcement
- Chunk verification

**Keep as-is**:
- Worker thread spawn loop
- Chunk queue management
- Retry count tracking
- Speed enforcement logic
- Byte counting on completion
- Progress reporting

---

## Execution Order

1. ✅ Create directory structure
2. ✅ Extract `network/client.rs` → Test
3. ✅ Extract `network/headers.rs` → Test
4. ✅ Extract `utils/filesystem.rs` → Test
5. ✅ Extract `core/types.rs` → Test
6. ✅ Extract `core/integrity.rs` → Test
7. ✅ Update `lib.rs` to use modules → Test
8. ✅ Final integration test (large file download)
9. ✅ Performance verification
10. ✅ Commit to git

---

## Rollback Plan

If ANY test fails:
1. `git checkout src-tauri/src/`
2. Identify issue
3. Fix in isolation
4. Re-test before proceeding

---

## Success Criteria

- [ ] All modules created
- [ ] `lib.rs` reduced to < 150 lines
- [ ] All tests pass
- [ ] Download completes 100%
- [ ] Speed maintained (18-24 MB/s)
- [ ] No compilation warnings
- [ ] Code documented with rustdoc

---

**Status**: Ready to execute  
**Estimated Time**: 2-3 hours  
**Risk Level**: LOW (with incremental approach)
