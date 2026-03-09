import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { join } from '@tauri-apps/api/path';
import { useDownloadStore } from '../../stores/downloadStore';
import { useUIStore } from '../../stores/uiStore';
import './AddDownloadModal.css';

export const AddDownloadModal = () => {
    const { showAddModal, setShowAddModal, defaultDownloadPath, initDefaultDownloadPath } = useUIStore();

    // Ensure default path is initialized from system Downloads dir if not yet set
    useEffect(() => { initDefaultDownloadPath(); }, []);
    const { pendingRequest, setPendingRequest, addDownload, setUrl, setSavePath, setThreads, threads } = useDownloadStore();

    const [url, setLocalUrl] = useState('');
    const [savePath, setLocalSavePath] = useState('');
    const [filename, setLocalFilename] = useState('');
    const [localThreads, setLocalThreads] = useState(16);
    const [loading, setLoading] = useState<'detect' | 'queue' | 'download' | null>(null);
    const [detectedSize, setDetectedSize] = useState<number | null>(null);
    const urlInputRef = useRef<HTMLInputElement>(null);

    const isOpen = showAddModal || !!pendingRequest;

    // Auto-fill from pending IPC request
    useEffect(() => {
        if (pendingRequest) {
            setLocalUrl(pendingRequest.url);
            setLocalFilename(pendingRequest.filename);
            setDetectedSize(pendingRequest.size ?? null);
            initSavePath(pendingRequest.filename);
        }
    }, [pendingRequest]);

    // Auto-paste from clipboard when modal opens
    useEffect(() => {
        if (isOpen && !pendingRequest) {
            setLocalUrl('');
            setLocalFilename('');
            setDetectedSize(null);
            setLocalThreads(threads);
            navigator.clipboard.readText().then(text => {
                if (text.startsWith('http://') || text.startsWith('https://')) {
                    setLocalUrl(text);
                }
            }).catch(() => { });
            initSavePath('');
            setTimeout(() => urlInputRef.current?.select(), 80);
        }
    }, [isOpen]);

    const initSavePath = async (fname: string) => {
        try {
            // Use the user-configured default path (falls back to system default if empty)
            const baseDir = defaultDownloadPath || '.';
            setLocalSavePath(fname ? await join(baseDir, fname) : baseDir);
        } catch { }
    };

    const handleClose = () => {
        setShowAddModal(false);
        setPendingRequest(null);
    };

    const handleDetect = async () => {
        if (!url.trim()) return;
        setLoading('detect');
        try {
            const [name, size] = await invoke<[string, number]>('get_file_details', { url });
            if (name) setLocalFilename(name);
            if (size) setDetectedSize(size);
        } catch (e) {
            console.warn('Could not detect:', e);
        } finally {
            setLoading(null);
        }
    };

    const handleBrowse = async () => {
        const selected = await open({
            directory: true,
            defaultPath: savePath,
        });
        if (selected && typeof selected === 'string') {
            const fname = filename || url.split('/').pop() || 'download';
            setLocalSavePath(await join(selected, fname));
        }
    };

    const startDownload = async (queue = false) => {
        if (!url.trim()) return;
        const fname = filename || url.split('/').pop() || 'download';
        let fullPath = savePath;
        if (!fullPath) {
            const baseDir = defaultDownloadPath || '.';
            fullPath = await join(baseDir, fname);
        }

        const dlId = crypto.randomUUID();
        const headers = pendingRequest?.headers ?? {};
        const referrer = pendingRequest?.referrer ?? null;

        // Add to store immediately for optimistic UI
        addDownload({
            id: dlId,
            filename: fname,
            url,
            savePath: fullPath,
            progress: 0,
            speed: 0,
            eta: 0,
            totalSize: detectedSize ?? 0,
            downloaded: 0,
            status: queue ? 'queued' : 'active',
            headers,
            referrer,
            addedAt: Date.now(),
        });

        // Sync legacy store
        setUrl(url);
        setSavePath(fullPath);
        setThreads(localThreads);

        handleClose();

        if (!queue) {
            try {
                await invoke('download_file', {
                    url,
                    filepath: fullPath,
                    threads: localThreads,
                    headers,
                    referrer,
                });
            } catch (e) {
                console.error('Download failed:', e);
            }
        }
    };

    const formatBytes = (b: number | null) => {
        if (!b) return null;
        const units = ['B', 'KB', 'MB', 'GB'];
        let v = b; let i = 0;
        while (v >= 1024 && i < units.length - 1) { v /= 1024; i++; }
        return `${v.toFixed(1)} ${units[i]}`;
    };

    if (!isOpen) return null;

    return (
        <div className="modal-overlay" onClick={(e) => e.target === e.currentTarget && handleClose()}>
            <div className="modal-box" onClick={e => e.stopPropagation()}>
                {/* Header */}
                <div className="modal-header">
                    <span className="modal-title">Add Download</span>
                    <button className="modal-close" onClick={handleClose}>✕</button>
                </div>

                {/* Body */}
                <div className="modal-body">
                    {/* URL Row */}
                    <div className="modal-field">
                        <label className="modal-label">URL Input</label>
                        <div className="modal-url-row">
                            <input
                                ref={urlInputRef}
                                className="modal-input"
                                value={url}
                                onChange={e => setLocalUrl(e.target.value)}
                                placeholder="https://..."
                                spellCheck={false}
                                onBlur={handleDetect}
                            />
                        </div>
                        {detectedSize && (
                            <span className="modal-hint">Detected size: {formatBytes(detectedSize)}</span>
                        )}
                    </div>

                    {/* Options section */}
                    <div className="modal-section-label">Options</div>

                    {/* Save To */}
                    <div className="modal-field">
                        <label className="modal-label">Save to:</label>
                        <div className="modal-path-row">
                            <input
                                className="modal-input modal-input-path"
                                value={savePath}
                                onChange={e => setLocalSavePath(e.target.value)}
                                readOnly={false}
                            />
                            <button className="modal-browse-btn" onClick={handleBrowse} title="Browse folder">
                                📁
                            </button>
                        </div>
                    </div>

                    {/* Filename */}
                    <div className="modal-field">
                        <label className="modal-label">Rename:</label>
                        <input
                            className="modal-input"
                            value={filename}
                            onChange={e => setLocalFilename(e.target.value)}
                            placeholder="filename.ext"
                        />
                    </div>

                    {/* Threads Slider */}
                    <div className="modal-field">
                        <label className="modal-label">
                            Threads (1–32)
                            <span className="modal-thread-count">{localThreads}</span>
                        </label>
                        <input
                            type="range"
                            min={1}
                            max={32}
                            value={localThreads}
                            onChange={e => setLocalThreads(Number(e.target.value))}
                            className="modal-slider"
                        />
                    </div>
                </div>

                {/* Footer Buttons */}
                <div className="modal-footer">
                    <button
                        className="modal-btn modal-btn-primary"
                        onClick={() => startDownload(false)}
                        disabled={!url.trim() || loading !== null}
                    >
                        {loading === 'download' ? '⏳ Starting…' : 'Download Now'}
                    </button>
                    <button
                        className="modal-btn modal-btn-secondary"
                        onClick={() => startDownload(true)}
                        disabled={!url.trim() || loading !== null}
                    >
                        Add to Queue
                    </button>
                </div>
            </div>
        </div>
    );
};
