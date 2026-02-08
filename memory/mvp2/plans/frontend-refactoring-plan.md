# Frontend Refactoring Plan

**Created**: 2026-02-08  
**Status**: Planning  
**Priority**: Critical - Foundation for MVP2 features

---

## Current State Analysis

### File Structure
```
src/
├── App.tsx (183 lines) - All logic in one file
├── App.css (styling)
├── main.tsx (entry point)
└── assets/
```

### Current Functionality in App.tsx
1. **State Management** (lines 16-22):
   - URL input
   - Save path
   - Progress tracking
   - Total size
   - Status messages
   - Thread count
   - Download history

2. **Event Listeners** (lines 24-40):
   - `download-progress` event
   - `download-start` event
   - LocalStorage history loading

3. **Core Functions**:
   - `browseFile()` - File dialog + filename detection
   - `startDownload()` - Invoke Rust backend
   - `addToHistory()` - Save to localStorage

4. **UI Components** (inline):
   - URL input field
   - File browser button
   - Thread slider
   - Start button
   - Progress bar
   - Status display
   - History list

### Dependencies
- React 19.1.0
- @tauri-apps/api v2
- @tauri-apps/plugin-dialog v2.6.0
- TypeScript 5.8.3

---

## Refactoring Goals

### ✅ Must Preserve
1. All existing functionality (download, progress, history)
2. Tauri event listeners
3. LocalStorage history persistence
4. File dialog integration
5. Filename auto-detection
6. Thread configuration

### 🎯 Improvements
1. **Component Organization**: Extract reusable components
2. **State Management**: Add Zustand for scalable state
3. **Type Safety**: Improve TypeScript types
4. **Code Reusability**: Create custom hooks
5. **Maintainability**: Separate concerns (UI, logic, state)

---

## Proposed Structure

```
src/
├── components/
│   ├── common/
│   │   ├── Button.tsx
│   │   ├── Input.tsx
│   │   └── ProgressBar.tsx
│   ├── download/
│   │   ├── DownloadForm.tsx
│   │   ├── DownloadControls.tsx
│   │   └── DownloadStatus.tsx
│   └── history/
│       ├── HistoryList.tsx
│       └── HistoryItem.tsx
├── hooks/
│   ├── useDownload.ts
│   ├── useTauriEvents.ts
│   └── useHistory.ts
├── stores/
│   ├── downloadStore.ts
│   └── historyStore.ts
├── types/
│   └── index.ts
├── utils/
│   ├── formatters.ts (formatBytes, formatSpeed)
│   └── storage.ts (localStorage helpers)
├── App.tsx (orchestrator)
└── main.tsx
```

---

## Implementation Phases

### Phase 1: Setup Dependencies ✅ SAFE
**Goal**: Add required packages without touching existing code

**Actions**:
1. Add Zustand for state management
2. Verify build still works

**Commands**:
```bash
npm install zustand
npm run build
```

**Risk**: Low - Only adding dependencies

---

### Phase 2: Extract Types ✅ SAFE
**Goal**: Create shared TypeScript types

**New Files**:
- `src/types/index.ts`

**Types to Extract**:
```typescript
export interface HistoryItem {
  id: number;
  url: string;
  filename: string;
  size: number;
  date: string;
  status: 'Success' | 'Failed';
}

export interface DownloadProgress {
  downloaded: number;
  total: number;
  percentage: number;
}

export interface DownloadConfig {
  url: string;
  savePath: string;
  threads: number;
}
```

**Risk**: Low - No existing code changes

---

### Phase 3: Create Utility Functions ✅ SAFE
**Goal**: Extract formatting logic

**New Files**:
- `src/utils/formatters.ts`
- `src/utils/storage.ts`

**Functions**:
```typescript
// formatters.ts
export const formatBytes = (bytes: number): string
export const formatSpeed = (bytesPerSecond: number): string
export const formatTime = (seconds: number): string

// storage.ts
export const saveHistory = (history: HistoryItem[]): void
export const loadHistory = (): HistoryItem[]
```

**Risk**: Low - Pure functions, no side effects

---

### Phase 4: Create Zustand Stores ⚠️ MODERATE
**Goal**: Centralize state management

**New Files**:
- `src/stores/downloadStore.ts`
- `src/stores/historyStore.ts`

**downloadStore.ts**:
```typescript
interface DownloadState {
  url: string;
  savePath: string;
  progress: number;
  totalSize: number;
  status: string;
  threads: number;
  setUrl: (url: string) => void;
  setSavePath: (path: string) => void;
  setProgress: (progress: number) => void;
  setTotalSize: (size: number) => void;
  setStatus: (status: string) => void;
  setThreads: (threads: number) => void;
}
```

**Migration Strategy**:
1. Create stores with same state structure
2. Keep useState in App.tsx initially
3. Gradually replace useState with store
4. Test after each replacement

**Risk**: Moderate - Changing state management

---

### Phase 5: Extract Custom Hooks ⚠️ MODERATE
**Goal**: Encapsulate logic

**New Files**:
- `src/hooks/useDownload.ts`
- `src/hooks/useTauriEvents.ts`
- `src/hooks/useHistory.ts`

**useDownload.ts**:
```typescript
export const useDownload = () => {
  const browseFile = async () => { /* existing logic */ };
  const startDownload = async () => { /* existing logic */ };
  return { browseFile, startDownload };
};
```

**useTauriEvents.ts**:
```typescript
export const useTauriEvents = () => {
  useEffect(() => {
    // Setup event listeners
    // Return cleanup
  }, []);
};
```

**Risk**: Moderate - Moving logic

---

### Phase 6: Extract UI Components ✅ SAFE
**Goal**: Create reusable components

**Component Extraction Order** (safest first):

1. **ProgressBar.tsx** (lines 147-156)
   - Pure presentational component
   - Props: `progress`, `totalSize`

2. **HistoryItem.tsx** (lines 166-175)
   - Pure presentational component
   - Props: `item: HistoryItem`

3. **HistoryList.tsx** (lines 163-178)
   - Wraps HistoryItem
   - Props: `history: HistoryItem[]`

4. **DownloadControls.tsx** (lines 115-145)
   - URL input, file browser, thread slider, start button
   - Props: callbacks and state

5. **DownloadStatus.tsx** (lines 157-160)
   - Status text and size display
   - Props: `status`, `progress`, `totalSize`

**Risk**: Low - Components are presentational

---

### Phase 7: Refactor App.tsx 🔴 HIGH RISK
**Goal**: Use extracted components and hooks

**New App.tsx Structure**:
```typescript
function App() {
  // Use stores
  const downloadState = useDownloadStore();
  const historyState = useHistoryStore();
  
  // Use hooks
  useT auriEvents();
  const { browseFile, startDownload } = useDownload();
  
  return (
    <main>
      <h1>🏴‍☠️ Pirate Downloader</h1>
      <DownloadControls />
      <DownloadStatus />
      <ProgressBar />
      <HistoryList />
    </main>
  );
}
```

**Risk**: High - Major restructuring

---

## Testing Strategy

### After Each Phase
1. **Build Test**: `npm run build` must succeed
2. **Dev Test**: `npm run dev` must work
3. **Functionality Test**:
   - Enter URL
   - Browse for file
   - Adjust threads
   - Start download
   - Verify progress updates
   - Check history saves

### Regression Checklist
- [ ] URL input works
- [ ] File dialog opens
- [ ] Filename auto-detection works
- [ ] Thread slider updates
- [ ] Download starts
- [ ] Progress bar updates
- [ ] Status messages display
- [ ] History saves to localStorage
- [ ] History loads on refresh
- [ ] All Tauri events fire correctly

---

## Rollback Plan

### If Issues Arise
1. **Git**: Commit after each phase
2. **Revert**: `git checkout -- src/` to undo
3. **Incremental**: Only proceed if tests pass

### Safe Points
- After Phase 1: Dependencies added
- After Phase 2: Types created
- After Phase 3: Utils created
- After Phase 4: Stores created
- After Phase 5: Hooks created
- After Phase 6: Components created
- After Phase 7: App.tsx refactored

---

## Success Criteria

### Functional
- ✅ All existing features work identically
- ✅ No regressions in download functionality
- ✅ History persistence maintained
- ✅ Event listeners working

### Code Quality
- ✅ Components under 100 lines each
- ✅ Reusable components in `common/`
- ✅ Type-safe with TypeScript
- ✅ State management with Zustand
- ✅ Custom hooks for logic

### Maintainability
- ✅ Clear file structure
- ✅ Separation of concerns
- ✅ Easy to add new features
- ✅ Testable components

---

## Timeline

- **Phase 1**: 10 minutes (add dependencies)
- **Phase 2**: 15 minutes (create types)
- **Phase 3**: 20 minutes (create utils)
- **Phase 4**: 30 minutes (create stores)
- **Phase 5**: 30 minutes (create hooks)
- **Phase 6**: 45 minutes (extract components)
- **Phase 7**: 30 minutes (refactor App.tsx)

**Total Estimated Time**: 3 hours

---

## Notes

- **No Breaking Changes**: All functionality must work exactly as before
- **Incremental**: Commit after each phase
- **Test Frequently**: Run app after every change
- **User Approval**: Get approval before Phase 4+ (state management changes)
