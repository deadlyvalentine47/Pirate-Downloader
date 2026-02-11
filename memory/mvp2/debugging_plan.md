# Debugging Plan: Download Control Issues

## 1. Slow Pause Response
**Symptom:** "paused it took some time to pause but kept downloading"
**Cause:** Worker threads only check for pause/stop signals *between* chunks (every 4MB+ or network request). If a chunk takes time to download (slow network or large buffer), the worker won't pause immediately.
**Fix:**
- [x] Add `control.signal.load()` check inside the inner `while let Some(chunk)` loop in `lib.rs`.
- [x] Ensure immediate exit on signal (break inner loop, then break retry loop, then break main loop).

## 2. Resume Glitch ("Back to Paused")
**Symptom:** "when i pressed resume it went back to pause state"
**Cause:** Old "zombie" tasks from previous paused state were still running (due to slow pause) and checking signal *after* resume set it to 0. They continued alongside new task.
**Fix:**
- [x] Implemented **Generation ID** (`AtomicU32`) in `DownloadControl`.
- [x] Updated `run_download_task` to accept `generation` ID.
- [x] Updated workers to check `control.generation != gen` and exit immediately if mismatched.
- [x] Updated `resume_download` to increment generation before starting new task.

## 3. "Finished Downloading" instead of Pausing
**Symptom:** "then it finised downloading i dont think it really paused"
**Cause:** Zombie tasks (see #2) + Double workers processing chunks.
**Fix:**
- [x] Addressed by Generation ID fix (Zombie tasks usage) and Granular Pause check.
- [x] Added `download-state` 'completed' event emission in `run_download_task` to ensure UI updates correctly on finish.
- [x] Added `download-state` 'failed' event emission in `resume_download` error handler.

## 4. Integrity Check / Resume from 0%
**Symptom:** Resume sometimes starts with 0 interactions or data if state isn't loaded.
**Fix:** 
- [x] (Already applied) Initialize `DownloadControl` with `metadata.downloaded_bytes`.
- [ ] Verify this works in practice with the new fix.

## Execution Order
1. **Fix Slow Pause**: Modify `lib.rs` to check signals in the hot loop.
2. **Verify Resume State**: Review frontend `useDownload.ts` and backend event timing.
3. **Integrity Check**: Ensure `verify_download` logic respects resumed states.
