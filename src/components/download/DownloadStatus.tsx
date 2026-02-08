// Download Status Component
import { formatBytes } from '../../utils/formatters';

interface DownloadStatusProps {
    status: string;
    progress: number;
    totalSize: number;
    state?: 'idle' | 'active' | 'paused' | 'stopped' | 'completed' | 'failed' | 'cancelled';
}

export const DownloadStatus = ({ status, progress, totalSize, state }: DownloadStatusProps) => {
    const getStateBadge = () => {
        if (!state) return null;

        const badgeStyles: Record<string, { bg: string; text: string; label: string }> = {
            active: { bg: '#007acc', text: 'white', label: '⬇️ Downloading' },
            paused: { bg: '#ff9800', text: 'white', label: '⏸️ Paused' },
            stopped: { bg: '#6c757d', text: 'white', label: '⏹️ Stopped' },
            completed: { bg: '#28a745', text: 'white', label: '✅ Completed' },
            failed: { bg: '#dc3545', text: 'white', label: '❌ Failed' },
            cancelled: { bg: '#6c757d', text: 'white', label: '🚫 Cancelled' },
        };

        const badge = badgeStyles[state];
        if (!badge) return null;

        return (
            <span style={{
                display: 'inline-block',
                padding: '4px 12px',
                borderRadius: '12px',
                fontSize: '12px',
                fontWeight: 600,
                background: badge.bg,
                color: badge.text,
                marginLeft: '10px',
            }}>
                {badge.label}
            </span>
        );
    };

    return (
        <div style={{ textAlign: 'center' }}>
            <strong>{status}</strong>
            {totalSize > 0 && ` (${formatBytes(progress)} / ${formatBytes(totalSize)})`}
            {getStateBadge()}
        </div>
    );
};
