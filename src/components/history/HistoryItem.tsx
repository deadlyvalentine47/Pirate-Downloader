// History Item Component
import type { HistoryItem as HistoryItemType } from '../../types';

interface HistoryItemProps {
    item: HistoryItemType;
}

export const HistoryItem = ({ item }: HistoryItemProps) => {
    return (
        <div style={{
            padding: '10px',
            borderBottom: '1px solid #eee',
            display: 'flex',
            justifyContent: 'space-between'
        }}>
            <div>
                <div style={{ fontWeight: 'bold' }}>{item.filename}</div>
                <div style={{ fontSize: '0.8em', color: '#666' }}>{item.url}</div>
            </div>
            <div style={{ textAlign: 'right' }}>
                <div style={{ color: item.status === 'Success' ? 'green' : 'red' }}>
                    {item.status}
                </div>
                <div style={{ fontSize: '0.8em', color: '#999' }}>
                    {item.size > 0 ? (item.size / 1024 / 1024).toFixed(1) + ' MB' : ''}
                </div>
            </div>
        </div>
    );
};
