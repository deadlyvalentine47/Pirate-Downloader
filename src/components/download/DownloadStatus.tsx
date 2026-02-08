// Download Status Component
import { formatBytes } from '../../utils/formatters';

interface DownloadStatusProps {
    status: string;
    progress: number;
    totalSize: number;
}

export const DownloadStatus = ({ status, progress, totalSize }: DownloadStatusProps) => {
    return (
        <div style={{ textAlign: 'center' }}>
            <strong>{status}</strong>
            {totalSize > 0 && ` (${formatBytes(progress)} / ${formatBytes(totalSize)})`}
        </div>
    );
};
