// Download state management with Zustand
import { create } from 'zustand';
import type { DownloadState, DownloadEntry, DownloadStatus } from '../types';

type LegacyDownloadStateType = 'idle' | 'active' | 'paused' | 'stopped' | 'completed' | 'failed' | 'cancelled';

interface DownloadStore extends DownloadState {
    // Legacy single-download state (kept for backward compat with useDownload hook)
    downloadId: string | null;
    downloadState: LegacyDownloadStateType;

    // IPC Request State
    pendingRequest: {
        url: string;
        filename: string;
        size?: number;
        headers?: Record<string, string>;
        referrer?: string | null;
    } | null;

    // *** New: unified downloads list ***
    downloads: DownloadEntry[];

    // Legacy Actions
    setUrl: (url: string) => void;
    setSavePath: (path: string) => void;
    setProgress: (progress: number) => void;
    setTotalSize: (size: number) => void;
    setStatus: (status: string) => void;
    setThreads: (threads: number) => void;
    setDownloadId: (id: string | null) => void;
    setDownloadState: (state: LegacyDownloadStateType) => void;
    setPendingRequest: (req: {
        url: string;
        filename: string;
        size?: number;
        headers?: Record<string, string>;
        referrer?: string | null;
    } | null) => void;
    reset: () => void;

    // New: Downloads list actions
    addDownload: (entry: DownloadEntry) => void;
    updateDownload: (id: string, patch: Partial<DownloadEntry>) => void;
    removeDownload: (id: string) => void;
    clearCompleted: () => void;
    getDownload: (id: string) => DownloadEntry | undefined;
}

const initialState: DownloadState & {
    downloadId: string | null;
    downloadState: LegacyDownloadStateType;
    pendingRequest: {
        url: string;
        filename: string;
        size?: number;
        headers?: Record<string, string>;
        referrer?: string | null;
    } | null;
    downloads: DownloadEntry[];
} = {
    url: '',
    savePath: '',
    progress: 0,
    totalSize: 0,
    status: 'Idle',
    threads: 16,
    downloadId: null,
    downloadState: 'idle',
    pendingRequest: null,
    downloads: [],
};

export const useDownloadStore = create<DownloadStore>((set, get) => ({
    ...initialState,

    // Legacy actions
    setUrl: (url) => set({ url }),
    setSavePath: (path) => set({ savePath: path }),
    setProgress: (progress) => set({ progress }),
    setTotalSize: (size) => set({ totalSize: size }),
    setStatus: (status) => set({ status }),
    setThreads: (threads) => set({ threads }),
    setDownloadId: (id) => set({ downloadId: id }),
    setDownloadState: (state) => set({ downloadState: state }),
    setPendingRequest: (req) => set({ pendingRequest: req }),
    reset: () => set(initialState),

    // Downloads list actions
    addDownload: (entry) =>
        set((state) => ({ downloads: [...state.downloads, entry] })),

    updateDownload: (id, patch) =>
        set((state) => ({
            downloads: state.downloads.map((d) =>
                d.id === id ? { ...d, ...patch } : d
            ),
        })),

    removeDownload: (id) =>
        set((state) => ({
            downloads: state.downloads.filter((d) => d.id !== id),
        })),

    clearCompleted: () =>
        set((state) => ({
            downloads: state.downloads.filter(
                (d) => d.status !== 'completed' && d.status !== 'cancelled' && d.status !== 'failed'
            ),
        })),

    getDownload: (id) => get().downloads.find((d) => d.id === id),
}));

// Helper: map legacy state string → DownloadStatus
export function toDownloadStatus(s: string): DownloadStatus {
    switch (s) {
        case 'active': return 'active';
        case 'paused': return 'paused';
        case 'completed': return 'completed';
        case 'failed': return 'failed';
        case 'cancelled': return 'cancelled';
        case 'stopped': return 'paused';
        default: return 'queued';
    }
}
