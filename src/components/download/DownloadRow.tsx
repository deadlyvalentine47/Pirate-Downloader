import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { openPath } from '@tauri-apps/plugin-opener';
import type { DownloadEntry } from '../../types';
import { useDownloadStore } from '../../stores/downloadStore';
import { ContextMenu, type ContextMenuItem } from '../common/ContextMenu';
import './DownloadRow.css';

interface DownloadRowProps {
    entry: DownloadEntry;
}

function getFileIcon(filename: string): string {
    const ext = filename.split('.').pop()?.toLowerCase() ?? '';
    if (['mp4', 'mkv', 'avi', 'mov', 'webm', 'flv', 'm2ts'].includes(ext)) return '🎬';
    if (['mp3', 'flac', 'aac', 'wav', 'ogg', 'm4a'].includes(ext)) return '🎵';
    if (['zip', 'rar', '7z', 'tar', 'gz', 'bz2'].includes(ext)) return '📦';
    if (['pdf', 'doc', 'docx', 'txt', 'xlsx', 'pptx'].includes(ext)) return '📄';
    if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'bmp'].includes(ext)) return '🖼';
    if (['exe', 'msi', 'dmg', 'deb', 'rpm', 'apk'].includes(ext)) return '💾';
    if (['m3u8', 'mpd'].includes(ext)) return '📡';
    return '📁';
}

function formatBytes(bytes: number): string {
    if (!bytes || bytes === 0) return '--';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let v = bytes;
    let i = 0;
    while (v >= 1024 && i < units.length - 1) { v /= 1024; i++; }
    return `${v.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

function formatSpeed(bps: number): string {
    if (!bps || bps === 0) return '--';
    return formatBytes(bps) + '/s';
}

function formatEta(seconds: number): string {
    if (!seconds || seconds <= 0) return '--';
    if (seconds < 60) return `${Math.round(seconds)}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${Math.round(seconds % 60)}s`;
    return `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;
}

function getStatusLabel(status: DownloadEntry['status']): { label: string; cls: string } {
    switch (status) {
        case 'active': return { label: 'Downloading', cls: 'status-active' };
        case 'paused': return { label: 'Paused', cls: 'status-paused' };
        case 'queued': return { label: 'Queued', cls: 'status-paused' };
        case 'completed': return { label: 'Completed', cls: 'status-done' };
        case 'failed': return { label: 'Failed', cls: 'status-error' };
        case 'cancelled': return { label: 'Cancelled', cls: 'status-error' };
        case 'waiting_for_link': return { label: 'Waiting…', cls: 'status-paused' };
        default: return { label: status, cls: '' };
    }
}

export const DownloadRow = ({ entry }: DownloadRowProps) => {
    const { updateDownload, removeDownload } = useDownloadStore();
    const [ctxMenu, setCtxMenu] = useState<{ x: number; y: number } | null>(null);

    const { label, cls } = getStatusLabel(entry.status);
    const isActive = entry.status === 'active';
    const isPaused = entry.status === 'paused' || entry.status === 'queued';

    const handleContextMenu = (e: React.MouseEvent) => {
        e.preventDefault();
        setCtxMenu({ x: e.clientX, y: e.clientY });
    };

    const handlePause = async () => {
        try {
            await invoke('pause_download', { downloadId: entry.id });
            updateDownload(entry.id, { status: 'paused' });
        } catch (e) { console.error(e); }
    };

    const handleResume = async () => {
        try {
            await invoke('resume_download', { downloadId: entry.id });
            updateDownload(entry.id, { status: 'active' });
        } catch (e) { console.error(e); }
    };

    const handleCancel = async () => {
        try {
            await invoke('cancel_download', { downloadId: entry.id });
            removeDownload(entry.id);
        } catch (e) { console.error(e); }
    };

    const openFolder = async () => {
        try {
            const folder = entry.savePath.replace(/[/\\][^/\\]+$/, '');
            await openPath(folder);
        } catch (e) { console.error(e); }
    };

    const copyUrl = () => navigator.clipboard.writeText(entry.url);

    const ctxItems: ContextMenuItem[] = [
        ...(isActive ? [{ label: 'Pause', icon: '⏸', onClick: handlePause }] : []),
        ...(isPaused ? [{ label: 'Resume', icon: '▶', onClick: handleResume }] : []),
        { label: 'Open Folder', icon: '📂', onClick: openFolder },
        { label: 'Copy URL', icon: '📋', onClick: copyUrl },
        { divider: true } as any,
        { label: 'Remove', icon: '🗑', onClick: handleCancel, danger: true },
    ];

    const progressColor = entry.status === 'paused' ? 'var(--accent-gold)'
        : entry.status === 'completed' ? 'var(--accent-green)'
            : entry.status === 'failed' ? 'var(--accent-red)'
                : 'var(--accent-blue)';

    return (
        <>
            <tr
                className={`dl-row ${isActive ? 'dl-row-active' : ''}`}
                onContextMenu={handleContextMenu}
            >
                {/* Icon + Name */}
                <td className="dl-cell dl-cell-name">
                    <span className="dl-icon">{getFileIcon(entry.filename)}</span>
                    <span className="dl-name truncate" title={entry.filename}>
                        {entry.filename}
                    </span>
                </td>

                {/* Size */}
                <td className="dl-cell dl-cell-size">
                    {formatBytes(entry.totalSize)}
                </td>

                {/* Progress */}
                <td className="dl-cell dl-cell-progress">
                    <div className="dl-progress-wrap">
                        <div className="dl-progress-bar-bg">
                            <div
                                className={`dl-progress-bar-fill ${isActive ? 'dl-progress-animated' : ''}`}
                                style={{
                                    width: `${Math.min(entry.progress, 100)}%`,
                                    background: progressColor,
                                }}
                            />
                        </div>
                        <span className="dl-progress-pct">
                            {entry.progress > 0 ? `${Math.round(entry.progress)}%` : '0%'}
                        </span>
                    </div>
                </td>

                {/* Speed */}
                <td className="dl-cell dl-cell-metric">
                    {isActive ? formatSpeed(entry.speed) : '--'}
                </td>

                {/* ETA */}
                <td className="dl-cell dl-cell-metric">
                    {entry.status === 'completed' ? '--' : isActive ? formatEta(entry.eta) : '--'}
                </td>

                {/* Status */}
                <td className="dl-cell dl-cell-status">
                    <span className={`dl-status-badge ${cls}`}>{label}</span>
                </td>
            </tr>

            {ctxMenu && (
                <ContextMenu
                    x={ctxMenu.x}
                    y={ctxMenu.y}
                    items={ctxItems}
                    onClose={() => setCtxMenu(null)}
                />
            )}
        </>
    );
};
