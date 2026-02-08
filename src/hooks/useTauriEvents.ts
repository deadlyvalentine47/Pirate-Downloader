// Custom hook for Tauri event listeners
import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useDownloadStore } from '../stores/downloadStore';

export const useTauriEvents = () => {
    const { setProgress, setTotalSize } = useDownloadStore();

    useEffect(() => {
        // Listen for download progress updates
        const unlistenProgress = listen<number>('download-progress', (event) => {
            setProgress(event.payload);
        });

        // Listen for download start (total size)
        const unlistenStart = listen<number>('download-start', (event) => {
            setTotalSize(event.payload);
        });

        // Cleanup listeners on unmount
        return () => {
            unlistenProgress.then(f => f());
            unlistenStart.then(f => f());
        };
    }, [setProgress, setTotalSize]);
};
