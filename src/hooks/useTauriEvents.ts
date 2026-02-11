// Custom hook for Tauri event listeners
import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useDownloadStore } from '../stores/downloadStore';

export const useTauriEvents = () => {
    const { setProgress, setTotalSize, setDownloadId, setDownloadState } = useDownloadStore();

    useEffect(() => {
        // Listen for download progress updates
        const unlistenProgress = listen<number>('download-progress', (event) => {
            setProgress(event.payload);
        });

        // Listen for download start (total size)
        const unlistenStart = listen<number>('download-start', (event) => {
            setTotalSize(event.payload);
        });

        // Listen for download ID
        const unlistenId = listen<string>('download-id', (event) => {
            setDownloadId(event.payload);
            setDownloadState('active');
        });

        // Listen for state changes
        const unlistenState = listen<string>('download-state', (event) => {
            setDownloadState(event.payload as any);
        });

        // Listen for IPC confirmation requests
        const unlistenConfirmation = listen<{ url: string, filename: string, size?: number }>('request-download-confirmation', (event) => {
            // Set pending request to trigger modal
            useDownloadStore.getState().setPendingRequest(event.payload);
        });

        // Cleanup listeners on unmount
        return () => {
            unlistenProgress.then(f => f());
            unlistenStart.then(f => f());
            unlistenId.then(f => f());
            unlistenState.then(f => f());
            unlistenConfirmation.then(f => f());
        };
    }, [setProgress, setTotalSize, setDownloadId, setDownloadState]);
};
