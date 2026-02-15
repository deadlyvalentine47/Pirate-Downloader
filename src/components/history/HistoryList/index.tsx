// History List Component
import type { HistoryItem as HistoryItemType } from '../../../types';
import { HistoryItem } from '../HistoryItem';
import './style.css';

interface HistoryListProps {
    history: HistoryItemType[];
}

export const HistoryList = ({ history }: HistoryListProps) => {
    return (
        <div className="history-section">
            <h2 className="history-list-title">📜 Download History</h2>
            <div className="history-table-container">
                {history.length > 0 ? (
                    <table className="history-table">
                        <thead>
                            <tr>
                                <th>Name</th>
                                <th>Status</th>
                                <th>Size</th>
                                <th>Date</th>
                                <th>Actions</th>
                            </tr>
                        </thead>
                        <tbody>
                            {history.map((item) => (
                                <HistoryItem key={item.id} item={item} />
                            ))}
                        </tbody>
                    </table>
                ) : (
                    <p className="history-empty-message">
                        No downloads yet. Your plunder will appear here.
                    </p>
                )}
            </div>
        </div>
    );
};
