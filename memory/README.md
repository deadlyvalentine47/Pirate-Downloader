# Memory Folder

This folder contains the **single source of truth** for Pirate Downloader's development roadmap and feature planning.

## Purpose

- 📋 **Central Documentation**: All PRDs, feature specs, and planning docs live here
- 🔄 **Always Updated**: This is the ONLY place we update when features change
- 🎯 **Reference Point**: Refer to these docs for implementation details

## Files

### `mvp2_prd.md`
**Product Requirements Document for MVP2**

Contains:
- 20 essential features across 4 priority levels
- Technical requirements for each feature
- Database schema
- UI/UX design principles
- 10-week development roadmap
- Competitive analysis

**Status**: Draft - Awaiting Approval  
**Last Updated**: 2026-02-05

---

### `educational_guide.md`
**Educational Guide: Multi-threaded Download Manager**

Deep dive into the technical implementation:
- How multi-threading works
- Chunk-based downloading strategy
- Retry mechanisms
- Speed enforcement
- Integrity verification

**Purpose**: Understanding the core download engine

---

### `implementation_guide.md`
**Step-by-Step Implementation Guide**

Detailed walkthrough of building the download manager:
- File allocation strategies
- Dynamic chunking
- Thread management
- Error handling
- Performance optimization

**Purpose**: Reference for implementation details

---

### `mvp1_walkthrough.md`
**MVP1 Completion Walkthrough**

Summary of what was accomplished in MVP1:
- Bug fixes (premature exit, infinite loops, false completions)
- Performance metrics
- Test results
- Lessons learned

**Purpose**: Historical record of MVP1 development

---

## Update Policy

✅ **DO**: Update files here when features are added, changed, or completed  
❌ **DON'T**: Create duplicate docs elsewhere  
📝 **ALWAYS**: Mark features as complete when implemented  

---

## Quick Reference

**Current Phase**: MVP1 Complete ✅  
**Next Phase**: MVP2 Phase 1 (Foundation)  
**Priority 1 Features**: Pause/Resume, Queue, Auto-Filename, History, Settings
