import React from 'react';
import { useDownloadStore } from '../../stores/downloadStore';
import { ConfirmDialog } from '../common/ConfirmDialog';
import { invoke } from '@tauri-apps/api/core';
import { downloadDir, join } from '@tauri-apps/api/path';
import { useHistoryStore } from '../../stores/historyStore';

export const IPCConfirmation = () => {
    const { pendingRequest, setPendingRequest, setUrl, setSavePath, setStatus } = useDownloadStore();
    const { addItem } = useHistoryStore();

    const handleConfirm = async () => {
        if (!pendingRequest) return;

        try {
            const downloads = await downloadDir();
            const fullPath = await join(downloads, pendingRequest.filename);

            // Set store state so UI reflects it
            setUrl(pendingRequest.url);
            setSavePath(fullPath);
            setStatus('Starting...');

            // Invoke download directly since we have all params
            // We use the same 'download_file' command
            invoke<{ status: string }>('download_file', {
                url: pendingRequest.url,
                filepath: fullPath,
                threads: 16 // Default
            }).then(() => {
                setStatus('Finished');
                addItem(pendingRequest.url, fullPath, 0, 'Success');
            }).catch(e => {
                setStatus('Error: ' + e);
                addItem(pendingRequest.url, fullPath, 0, 'Failed');
            });

        } catch (e) {
            console.error('Failed to resolve path', e);
        }

        setPendingRequest(null);
    };

    const handleCancel = () => {
        setPendingRequest(null);
    };

    if (!pendingRequest) return null;

    const formatSize = (bytes?: number) => {
        if (!bytes || bytes === 0) return 'Unknown size';
        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        let size = bytes;
        let unitIndex = 0;
        while (size >= 1024 && unitIndex < units.length - 1) {
            size /= 1024;
            unitIndex++;
        }
        return `${size.toFixed(2)} ${units[unitIndex]}`;
    };

    const displaySize = formatSize(pendingRequest.size);

    return (
        <ConfirmDialog
            open={!!pendingRequest}
            title="Download Request 🏴‍☠️"
            message={`Do you want to download:\n${pendingRequest.filename}\n\nSize: ${displaySize}\n\nFrom:\n${pendingRequest.url.substring(0, 50)}...`}
            confirmText="Download Now"
            cancelText="Cancel"
            onConfirm={handleConfirm}
            onCancel={handleCancel}
            variant="warning"
        />
    );
};
