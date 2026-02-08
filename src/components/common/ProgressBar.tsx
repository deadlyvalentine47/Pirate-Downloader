// Progress Bar Component
import { calculatePercentage } from '../../utils/formatters';

interface ProgressBarProps {
    progress: number;
    totalSize: number;
}

export const ProgressBar = ({ progress, totalSize }: ProgressBarProps) => {
    const percentage = calculatePercentage(progress, totalSize);

    if (progress === 0) return null;

    return (
        <div style={{
            background: '#e0e0e0',
            borderRadius: '10px',
            height: '20px',
            overflow: 'hidden'
        }}>
            <div style={{
                width: `${percentage}%`,
                background: '#4caf50',
                height: '100%',
                transition: 'width 0.2s'
            }} />
        </div>
    );
};
