// History Item Component
import type { HistoryItem as HistoryItemType } from '../../../types';
import './style.css';

interface HistoryItemProps {
    item: HistoryItemType;
}

export const HistoryItem = ({ item }: HistoryItemProps) => {
    return (
        <div className="history-item-container">
            <div className="history-item-info">
                <div className="history-filename">{item.filename}</div>
                <div className="history-url">{item.url}</div>
            </div>
            <div className="history-status-container">
                <div className={item.status === 'Success' ? 'status-success' : 'status-failed'}>
                    {item.status}
                </div>
                <div className="history-size">
                    {item.size > 0 ? (item.size / 1024 / 1024).toFixed(1) + ' MB' : ''}
                </div>
            </div>
        </div>
    );
};
