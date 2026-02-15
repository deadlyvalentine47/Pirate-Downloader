// History Item Component
import type { HistoryItem as HistoryItemType } from '../../../types';
import './style.css';

interface HistoryItemProps {
    item: HistoryItemType;
}

export const HistoryItem = ({ item }: HistoryItemProps) => {
    const formatDate = (dateStr?: string) => {
        if (!dateStr) return '-';
        const date = new Date(dateStr);
        return date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    };

    return (
        <tr className="history-row">
            <td className="history-cell-name" title={item.url}>
                <div className="history-filename">{item.filename}</div>
                <div className="history-url-sub">{item.url}</div>
            </td>
            <td className="history-cell-status">
                <span className={`status-badge ${item.status.toLowerCase()}`}>
                    {item.status}
                </span>
            </td>
            <td className="history-cell-size">
                {item.size > 0 ? (item.size / 1024 / 1024).toFixed(1) + ' MB' : 'Unknown'}
            </td>
            <td className="history-cell-date">
                {formatDate(item.timestamp)}
            </td>
            <td className="history-cell-actions">
                <button className="action-btn-mini" title="Open Folder">📂</button>
                <button className="action-btn-mini" title="Redownload">🔄</button>
            </td>
        </tr>
    );
};
