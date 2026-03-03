# MVP4 Tracker - Pirate Downloader

## Universal Streaming Engine [New Architecture]
*   [x] **Foundation & Strategy Hub:** Create `UniversalStreamingStrategy` and config toggles. [Status: Done]
*   [x] **Parallel Segment Downloader:** Implement worker pool and buffered ordered writing. [Status: Done]
*   [x] **Smart Processor (The .jpg Fix):** Implement MPEG-TS sync detection and header stripping. [Status: Done]
*   [x] **HLS/DASH Resolver:** Refactor existing HLS logic into the new Resolver layer. [Status: Done]
*   [x] **YouTube Platform Resolver:** Implement YouTube URL extraction logic. [Status: Done]
*   [ ] **Final Engine Integration:** Update `lib.rs` to route traffic to the new engine. [Status: In Progress]

## UI & UX Enhancements
*   [ ] **Progress Pulse Improvements:** Real-time segment status updates in the UI. [Status: Pending]
*   [ ] **Global Download Toggles:** Add UI settings to enable/disable specific engine features. [Status: Pending]

## Core Bug Fixes & Optimization
*   [ ] **Sparse File Allocation:** Ensure HLS/DASH files are pre-allocated correctly. [Status: Pending]
*   [ ] **Error Handling Refine:** Improved error messages for network timeouts vs. server blocks. [Status: Pending]
