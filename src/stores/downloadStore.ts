// Download state management with Zustand
import { create } from 'zustand';
import type { DownloadState } from '../types';

type DownloadStateType = 'idle' | 'active' | 'paused' | 'stopped' | 'completed' | 'failed' | 'cancelled';

interface DownloadStore extends DownloadState {
    // Download control state
    downloadId: string | null;
    downloadState: DownloadStateType;

    // Actions
    setUrl: (url: string) => void;
    setSavePath: (path: string) => void;
    setProgress: (progress: number) => void;
    setTotalSize: (size: number) => void;
    setStatus: (status: string) => void;
    setThreads: (threads: number) => void;
    setDownloadId: (id: string | null) => void;
    setDownloadState: (state: DownloadStateType) => void;
    reset: () => void;
}

const initialState: DownloadState & { downloadId: string | null; downloadState: DownloadStateType } = {
    url: '',
    savePath: '',
    progress: 0,
    totalSize: 0,
    status: 'Idle',
    threads: 16,
    downloadId: null,
    downloadState: 'idle',
};

export const useDownloadStore = create<DownloadStore>((set) => ({
    ...initialState,

    setUrl: (url) => set({ url }),
    setSavePath: (path) => set({ savePath: path }),
    setProgress: (progress) => set({ progress }),
    setTotalSize: (size) => set({ totalSize: size }),
    setStatus: (status) => set({ status }),
    setThreads: (threads) => set({ threads }),
    setDownloadId: (id) => set({ downloadId: id }),
    setDownloadState: (state) => set({ downloadState: state }),
    reset: () => set(initialState),
}));
