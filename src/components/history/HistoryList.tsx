// History List Component
import type { HistoryItem as HistoryItemType } from '../../types';
import { HistoryItem } from './HistoryItem';

interface HistoryListProps {
    history: HistoryItemType[];
}

export const HistoryList = ({ history }: HistoryListProps) => {
    return (
        <>
            <h2 style={{ marginTop: '30px' }}>📜 History</h2>
            <div style={{ borderTop: '2px solid #eee' }}>
                {history.map((item) => (
                    <HistoryItem key={item.id} item={item} />
                ))}
                {history.length === 0 && (
                    <p style={{ color: '#888', textAlign: 'center', padding: '20px' }}>
                        No downloads yet.
                    </p>
                )}
            </div>
        </>
    );
};
