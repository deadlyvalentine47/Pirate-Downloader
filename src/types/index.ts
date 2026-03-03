// Shared TypeScript types for Pirate Downloader

export interface HistoryItem {
    id: number;
    url: string;
    filename: string;
    size: number;
    timestamp: string;
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

export interface DownloadState {
    url: string;
    savePath: string;
    progress: number;
    totalSize: number;
    status: string;
    threads: number;
}

export type DownloadStatus = 'active' | 'paused' | 'queued' | 'completed' | 'failed' | 'cancelled' | 'waiting_for_link';

export interface DownloadEntry {
    id: string;
    filename: string;
    url: string;
    savePath: string;
    progress: number;       // 0–100
    speed: number;          // bytes/sec
    eta: number;            // seconds remaining
    totalSize: number;      // bytes
    downloaded: number;     // bytes
    status: DownloadStatus;
    headers?: Record<string, string>;
    referrer?: string | null;
    addedAt: number;        // timestamp ms
}
