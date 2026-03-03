// Custom hook for Tauri event listeners
import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useDownloadStore, toDownloadStatus } from '../stores/downloadStore';

export const useTauriEvents = () => {
    const {
        setProgress, setTotalSize, setDownloadId, setDownloadState,
        addDownload, updateDownload,
    } = useDownloadStore();

    useEffect(() => {
        // ── Legacy single-download progress ──────────────────────────
        const unlistenProgress = listen<number>('download-progress', (event) => {
            setProgress(event.payload);
            // Also propagate to the downloads list if there's an active entry
            const { downloadId, downloads } = useDownloadStore.getState();
            if (downloadId) {
                const entry = downloads.find(d => d.id === downloadId);
                if (entry) {
                    updateDownload(downloadId, { progress: event.payload });
                }
            }
        });

        // ── Download start (total size) ───────────────────────────────
        const unlistenStart = listen<number>('download-start', (event) => {
            setTotalSize(event.payload);
            const { downloadId } = useDownloadStore.getState();
            if (downloadId) {
                updateDownload(downloadId, { totalSize: event.payload });
            }
        });

        // ── Download ID assigned ──────────────────────────────────────
        const unlistenId = listen<string>('download-id', (event) => {
            const id = event.payload;
            setDownloadId(id);
            setDownloadState('active');

            // Create a new entry in the downloads list if not already there
            const { downloads, url, savePath } = useDownloadStore.getState();
            if (!downloads.find(d => d.id === id)) {
                const filename = savePath.split(/[\\/]/).pop() ?? 'Unknown File';
                addDownload({
                    id,
                    filename,
                    url,
                    savePath,
                    progress: 0,
                    speed: 0,
                    eta: 0,
                    totalSize: 0,
                    downloaded: 0,
                    status: 'active',
                    addedAt: Date.now(),
                });
            }
        });

        // ── Structured progress update (speed + eta) ─────────────────
        const unlistenDetailedProgress = listen<{
            id: string;
            progress: number;
            speed: number;
            eta: number;
            downloaded: number;
            total: number;
        }>('download-progress-detail', (event) => {
            const { id, progress, speed, eta, downloaded, total } = event.payload;
            updateDownload(id, { progress, speed, eta, downloaded, totalSize: total });
        });

        // ── State changes ─────────────────────────────────────────────
        const unlistenState = listen<string>('download-state', (event) => {
            setDownloadState(event.payload as any);
            const { downloadId } = useDownloadStore.getState();
            if (downloadId) {
                updateDownload(downloadId, { status: toDownloadStatus(event.payload) });
            }
        });

        // ── IPC confirmation requests ─────────────────────────────────
        const unlistenConfirmation = listen<{
            url: string;
            filename: string;
            size?: number;
            headers?: Record<string, string>;
            referrer?: string | null;
        }>('request-download-confirmation', (event) => {
            useDownloadStore.getState().setPendingRequest(event.payload);
        });

        // Cleanup
        return () => {
            unlistenProgress.then(f => f());
            unlistenStart.then(f => f());
            unlistenId.then(f => f());
            unlistenDetailedProgress.then(f => f());
            unlistenState.then(f => f());
            unlistenConfirmation.then(f => f());
        };
    }, [setProgress, setTotalSize, setDownloadId, setDownloadState, addDownload, updateDownload]);
};
