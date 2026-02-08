// Shared TypeScript types for Pirate Downloader

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

export interface DownloadState {
    url: string;
    savePath: string;
    progress: number;
    totalSize: number;
    status: string;
    threads: number;
}
