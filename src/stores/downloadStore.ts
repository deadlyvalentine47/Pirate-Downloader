// Download state management with Zustand
import { create } from 'zustand';
import type { DownloadState } from '../types';

interface DownloadStore extends DownloadState {
    // Actions
    setUrl: (url: string) => void;
    setSavePath: (path: string) => void;
    setProgress: (progress: number) => void;
    setTotalSize: (size: number) => void;
    setStatus: (status: string) => void;
    setThreads: (threads: number) => void;
    reset: () => void;
}

const initialState: DownloadState = {
    url: '',
    savePath: '',
    progress: 0,
    totalSize: 0,
    status: 'Idle',
    threads: 16,
};

export const useDownloadStore = create<DownloadStore>((set) => ({
    ...initialState,

    setUrl: (url) => set({ url }),
    setSavePath: (path) => set({ savePath: path }),
    setProgress: (progress) => set({ progress }),
    setTotalSize: (size) => set({ totalSize: size }),
    setStatus: (status) => set({ status }),
    setThreads: (threads) => set({ threads }),
    reset: () => set(initialState),
}));
