# Download Control Frontend Implementation Plan

**Feature**: Download Control UI (Pause/Resume/Stop/Cancel)  
**Status**: Planning  
**Backend**: ✅ Complete  
**Frontend**: 🔄 In Progress

---

## Overview

Add UI controls to allow users to pause, resume, stop, and cancel downloads. The UI should show appropriate controls based on download state and include confirmation dialogs for destructive actions.

---

## Components to Create/Modify

### 1. DownloadControls Component (NEW)
**Location**: `src/components/DownloadControls.tsx`

**Purpose**: Render state-appropriate control buttons

**Props**:
```typescript
interface DownloadControlsProps {
  downloadId: string;
  state: 'active' | 'paused' | 'stopped' | 'completed' | 'failed' | 'cancelled';
  onPause: () => void;
  onResume: () => void;
  onStop: () => void;
  onCancel: () => void;
}
```

**Buttons by State**:
- **Active**: Pause, Stop, Cancel
- **Paused**: Resume, Cancel
- **Stopped**: Resume, Cancel
- **Failed**: Retry, Cancel
- **Completed**: (none)
- **Cancelled**: (none)

### 2. ConfirmDialog Component (NEW)
**Location**: `src/components/ConfirmDialog.tsx`

**Purpose**: Show confirmation for destructive actions (Cancel)

**Props**:
```typescript
interface ConfirmDialogProps {
  open: boolean;
  title: string;
  message: string;
  confirmText: string;
  cancelText: string;
  onConfirm: () => void;
  onCancel: () => void;
  variant?: 'warning' | 'danger';
}
```

### 3. DownloadStatus Component (MODIFY)
**Location**: `src/components/DownloadStatus.tsx`

**Changes**:
- Add state badge/indicator
- Color-code by state (active=blue, paused=yellow, stopped=gray, failed=red, cancelled=gray)
- Show appropriate icon for each state

### 4. App.tsx (MODIFY)
**Location**: `src/App.tsx`

**Changes**:
- Import and use DownloadControls component
- Add handlers for pause/resume/stop/cancel
- Invoke Tauri commands
- Show ConfirmDialog for cancel action
- Update download state in store

---

## Tauri Command Integration

### Commands to Call

```typescript
import { invoke } from '@tauri-apps/api/core';

// Pause download
await invoke('pause_download', { downloadId: string });

// Resume download
await invoke('resume_download', { downloadId: string });

// Stop download
await invoke('stop_download', { downloadId: string });

// Cancel download (with confirmation)
await invoke('cancel_download', { downloadId: string });
```

### Error Handling

```typescript
try {
  await invoke('pause_download', { downloadId });
  // Update UI state
} catch (error) {
  console.error('Failed to pause download:', error);
  // Show error toast/notification
}
```

---

## State Management

### Update downloadStore

**Add to store**:
```typescript
interface DownloadStore {
  // ... existing fields
  downloadState: 'active' | 'paused' | 'stopped' | 'completed' | 'failed' | 'cancelled';
  
  // Actions
  pauseDownload: () => Promise<void>;
  resumeDownload: () => Promise<void>;
  stopDownload: () => Promise<void>;
  cancelDownload: () => Promise<void>;
  setDownloadState: (state: string) => void;
}
```

---

## UI/UX Considerations

### Button Styling
- **Pause**: Blue/Primary
- **Resume**: Green/Success
- **Stop**: Orange/Warning
- **Cancel**: Red/Danger
- **Retry**: Blue/Primary

### Confirmation Dialog
- **Title**: "Cancel Download?"
- **Message**: "This will delete the partial file and cannot be undone. Are you sure?"
- **Confirm Button**: "Yes, Cancel Download" (Red)
- **Cancel Button**: "No, Keep Download" (Gray)

### State Indicators
- **Active**: Blue badge with "Downloading..."
- **Paused**: Yellow badge with "Paused"
- **Stopped**: Gray badge with "Stopped"
- **Failed**: Red badge with "Failed"
- **Completed**: Green badge with "Completed"
- **Cancelled**: Gray badge with "Cancelled"

### Loading States
- Disable buttons while command is executing
- Show spinner on clicked button
- Re-enable after command completes

---

## Implementation Steps

1. **Create ConfirmDialog Component**
   - Reusable dialog for confirmations
   - Styled with warning/danger variants
   - Accessible (keyboard navigation, focus trap)

2. **Create DownloadControls Component**
   - Conditional rendering based on state
   - Button click handlers
   - Loading states
   - Tooltips for each button

3. **Update DownloadStatus Component**
   - Add state badge
   - Color-code states
   - Add state icons

4. **Update downloadStore**
   - Add state field
   - Add action methods
   - Integrate Tauri commands

5. **Update App.tsx**
   - Add DownloadControls to download UI
   - Wire up handlers
   - Add ConfirmDialog
   - Handle state updates

6. **Test All Flows**
   - Start → Pause → Resume
   - Start → Stop
   - Start → Cancel (with confirmation)
   - Pause → Cancel
   - Stop → Resume
   - Failed → Retry

---

## Files to Create

- `src/components/DownloadControls.tsx` (~100 lines)
- `src/components/ConfirmDialog.tsx` (~80 lines)

## Files to Modify

- `src/components/DownloadStatus.tsx` (+30 lines)
- `src/stores/downloadStore.ts` (+50 lines)
- `src/App.tsx` (+40 lines)

**Total**: ~300 lines of new/modified code

---

## Testing Checklist

- [ ] Pause button pauses active download
- [ ] Resume button resumes paused download
- [ ] Stop button stops active download
- [ ] Cancel shows confirmation dialog
- [ ] Confirmation dialog cancels download
- [ ] Buttons disabled during command execution
- [ ] State badge updates correctly
- [ ] Error handling shows user-friendly messages
- [ ] Keyboard navigation works in dialog
- [ ] All states render correct buttons

---

## Next Steps

1. Create ConfirmDialog component
2. Create DownloadControls component
3. Update DownloadStatus with state badges
4. Update downloadStore with actions
5. Integrate into App.tsx
6. Test all operations
