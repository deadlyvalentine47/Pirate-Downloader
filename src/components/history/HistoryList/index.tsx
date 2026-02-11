// History List Component
import type { HistoryItem as HistoryItemType } from '../../../types';
import { HistoryItem } from '../HistoryItem';
import './style.css';

interface HistoryListProps {
    history: HistoryItemType[];
}

export const HistoryList = ({ history }: HistoryListProps) => {
    return (
        <>
            <h2 className="history-list-title">📜 History</h2>
            <div className="history-list-container">
                {history.map((item) => (
                    <HistoryItem key={item.id} item={item} />
                ))}
                {history.length === 0 && (
                    <p className="history-empty-message">
                        No downloads yet.
                    </p>
                )}
            </div>
        </>
    );
};
