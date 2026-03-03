# HLS/DASH Robustness Specification

This document outlines the specific requirements for making the streaming engine robust against common web-streaming obstacles.

## 1. Parallel Segment Fetching
- Implementation of a worker pool for concurrent HLS/DASH segment downloading.
- Ordered buffered writing to maintain stream continuity.

## 2. Smart Header Stripping (The ".jpg" Fix)
- Automatic detection of non-video headers in segments.
- MPEG-TS Sync Byte (`0x47`) scanning and data cleaning.

## 3. Manifest Recovery & Retries
- Handling of expired segment URLs via re-fetching the manifest.
- Adaptive retry logic for 403/404 errors on individual segments.

## 4. Adaptive Stream Selection
- Logic to automatically select the highest available quality from a Master Playlist.
- Fallback mechanisms for broken sub-manifests.
