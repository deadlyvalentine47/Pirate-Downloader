import { useMemo } from 'react';
import { useDownloadStore } from '../../stores/downloadStore';
import { useHistoryStore } from '../../stores/historyStore';
import { useUIStore, type TabId } from '../../stores/uiStore';
import type { DownloadEntry } from '../../types';
import { DownloadRow } from './DownloadRow';
import './DownloadRow.css';

interface DownloadTableProps {
    filter: Exclude<TabId, 'settings'>;
}

export const DownloadTable = ({ filter }: DownloadTableProps) => {
    const { downloads } = useDownloadStore();
    const { history } = useHistoryStore();
    const { searchQuery } = useUIStore();

    // Merge history items as "completed" DownloadEntry objects
    const historyEntries: DownloadEntry[] = useMemo(() =>
        history.map(h => ({
            id: String(h.id),
            filename: h.filename || h.url.split('/').pop() || 'Unknown',
            url: h.url,
            savePath: h.filename || '',
            progress: 100,
            speed: 0,
            eta: 0,
            totalSize: h.size,
            downloaded: h.size,
            status: h.status === 'Success' ? 'completed' : 'failed',
            addedAt: new Date(h.timestamp).getTime(),
        } as DownloadEntry)),
        [history]);

    // Merge active downloads + history, dedupe by id
    const allEntries = useMemo(() => {
        const map = new Map<string, DownloadEntry>();
        // History first (lower priority)
        historyEntries.forEach(e => map.set(e.id, e));
        // Active downloads override
        downloads.forEach(e => map.set(e.id, e));
        return Array.from(map.values()).sort((a, b) => b.addedAt - a.addedAt);
    }, [downloads, historyEntries]);

    // Filter by tab
    const filtered = useMemo(() => {
        let entries = allEntries;
        switch (filter) {
            case 'downloading': entries = allEntries.filter(e => e.status === 'active'); break;
            case 'paused': entries = allEntries.filter(e => e.status === 'paused' || e.status === 'queued' || e.status === 'waiting_for_link'); break;
            case 'completed': entries = allEntries.filter(e => e.status === 'completed' || e.status === 'failed' || e.status === 'cancelled'); break;
            default: break;
        }
        // Search filter
        if (searchQuery.trim()) {
            const q = searchQuery.toLowerCase();
            entries = entries.filter(e => e.filename.toLowerCase().includes(q) || e.url.toLowerCase().includes(q));
        }
        return entries;
    }, [allEntries, filter, searchQuery]);

    if (filtered.length === 0) {
        return (
            <div className="empty-state">
                <div className="empty-state-icon">
                    {filter === 'downloading' ? '▶' : filter === 'paused' ? '⏸' : filter === 'completed' ? '✓' : '⬇'}
                </div>
                <p className="empty-state-text">
                    {filter === 'downloading' ? 'No active downloads. Start one from the sidebar!' :
                        filter === 'paused' ? 'No paused or queued downloads.' :
                            filter === 'completed' ? 'No completed downloads yet. Your plunder will appear here.' :
                                'No downloads yet. Click "+ Add New Download" to begin.'}
                </p>
            </div>
        );
    }

    return (
        <table className="dl-table">
            <thead>
                <tr>
                    <th>File Name</th>
                    <th>Size</th>
                    <th>Progress</th>
                    <th>Speed</th>
                    <th>ETA</th>
                    <th>Status</th>
                </tr>
            </thead>
            <tbody>
                {filtered.map(entry => (
                    <DownloadRow key={entry.id} entry={entry} />
                ))}
            </tbody>
        </table>
    );
};
