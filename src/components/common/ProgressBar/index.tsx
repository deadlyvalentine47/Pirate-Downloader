// Progress Bar Component
import { calculatePercentage } from '../../../utils/formatters';
import './style.css';

interface ProgressBarProps {
    progress: number;
    totalSize: number;
}

export const ProgressBar = ({ progress, totalSize }: ProgressBarProps) => {
    const percentage = calculatePercentage(progress, totalSize);

    if (progress === 0) return null;

    return (
        <div className="progress-bar-container">
            <div
                className="progress-bar-fill"
                style={{ width: `${percentage}%` }}
            />
        </div>
    );
};
