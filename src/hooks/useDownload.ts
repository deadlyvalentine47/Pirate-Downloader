// Custom hook for download operations
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { useDownloadStore } from '../stores/downloadStore';
import { useHistoryStore } from '../stores/historyStore';

export const useDownload = () => {
    const { url, savePath, threads, totalSize, setSavePath, setTotalSize, setStatus } = useDownloadStore();
    const { addItem } = useHistoryStore();

    /**
     * Open file dialog and detect filename from URL
     */
    const browseFile = async () => {
        let defaultFileName = 'download.dat';
        let detectedSize = 0;

        if (url) {
            setStatus('Checking URL...');
            try {
                // Ask Rust backend for file details
                const [name, size] = await invoke<[string, number]>('get_file_details', { url });
                defaultFileName = name;
                detectedSize = size;
                setTotalSize(detectedSize);
                setStatus('Detected: ' + name);
            } catch (e) {
                console.log('Could not detect filename', e);
            }
        }

        // Open save dialog
        const path = await save({
            defaultPath: defaultFileName,
            filters: [{ name: 'All Files', extensions: ['*'] }]
        });

        if (path) {
            setSavePath(path);
            if (detectedSize > 0) setTotalSize(detectedSize);
            setStatus('Ready');
        }
    };

    /**
     * Start the download
     */
    const startDownload = async () => {
        if (!url || !savePath) {
            alert('Please select a URL and a Save Path!');
            return;
        }

        setStatus('Downloading...');
        try {
            // Backend emits download-id event and returns result when complete/paused
            interface DownloadResult {
                id: string;
                status: string;
            }
            
            const result = await invoke<DownloadResult>('download_file', {
                url: url,
                filepath: savePath,
                threads: Number(threads)
            });

            if (result.status === 'completed') {
                setStatus('Finished');
                addItem(url, savePath, totalSize, 'Success');
            } else if (result.status === 'paused') {
                setStatus('Paused');
            } else if (result.status === 'stopped') {
                setStatus('Stopped');
            } else {
                setStatus('Cancelled');
            }
        } catch (e) {
            setStatus('Error: ' + e);
            addItem(url, savePath, 0, 'Failed');
            useDownloadStore.getState().setDownloadState('failed');
        }
    };

    return {
        browseFile,
        startDownload
    };
};
