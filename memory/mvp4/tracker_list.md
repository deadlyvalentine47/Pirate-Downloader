# MVP4: Feature Tracker List

This file tracks the high-level tasks for the "Peak Efficiency" phase of Pirate Downloader. Detailed specifications for each task are kept in separate files within the `memory/mvp4/` directory.

## 1. HLS Robustness (Phase 1)
*   [ ] **HLS Parallel Segment Downloading:** Implement a worker pool for HLS segments (similar to `HttpStrategy`). [Status: Pending] [Spec: `hls_robustness.md`]
*   [ ] **Master Playlist Resolver:** Enable recursive parsing for HLS master playlists and variant selection. [Status: Pending] [Spec: `hls_robustness.md`]
*   [ ] **Quality Selection UI:** Create a React modal for selecting between different HLS resolutions/bitrates. [Status: Pending] [Spec: `hls_robustness.md`]
*   [ ] **AES-128 Decryption:** Add native support for encrypted HLS streams. [Status: Pending] [Spec: `hls_robustness.md`]

## 2. Platform Expansion (Future)
*   [ ] **DASH Stream Robustness:** Port HLS improvements to the DASH downloader. [Status: Planned]
*   [ ] **Browser Extension - Firefox Support:** Ensure the native host works seamlessly with Firefox. [Status: Planned]

---
*Last Updated: March 2, 2026*
