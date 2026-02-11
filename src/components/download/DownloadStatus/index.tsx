// Download Status Component
import { formatBytes } from '../../../utils/formatters';
import './style.css';

interface DownloadStatusProps {
    status: string;
    progress: number;
    totalSize: number;
    state?: 'idle' | 'active' | 'paused' | 'stopped' | 'completed' | 'failed' | 'cancelled';
}

export const DownloadStatus = ({ status, progress, totalSize, state }: DownloadStatusProps) => {
    const getStateBadge = () => {
        if (!state) return null;

        const badgeLabels: Record<string, string> = {
            active: '⬇️ Downloading',
            paused: '⏸️ Paused',
            stopped: '⏹️ Stopped',
            completed: '✅ Completed',
            failed: '❌ Failed',
            cancelled: '🚫 Cancelled',
        };

        const label = badgeLabels[state];
        if (!label) return null;

        return (
            <span className={`status-badge badge-${state}`}>
                {label}
            </span>
        );
    };

    return (
        <div className="status-container">
            <strong>{status}</strong>
            {totalSize > 0 && ` (${formatBytes(progress)} / ${formatBytes(totalSize)})`}
            {getStateBadge()}
        </div>
    );
};
