import { invoke } from '@tauri-apps/api/core';
import { useDownloadStore } from '../../stores/downloadStore';
import { useHistoryStore } from '../../stores/historyStore';
import { useUIStore, type TabId } from '../../stores/uiStore';

interface TopBarConfig {
    buttons: {
        label: string;
        icon: string;
        onClick: () => void;
        disabled?: boolean;
    }[];
}

export const TopBar = () => {
    const { currentTab, searchQuery, setSearchQuery } = useUIStore();
    const { downloads, clearCompleted } = useDownloadStore();
    const { clearHistory } = useHistoryStore();

    const activeIds = downloads.filter(d => d.status === 'active').map(d => d.id);
    const pausedIds = downloads.filter(d => d.status === 'paused' || d.status === 'queued').map(d => d.id);
    const hasActive = activeIds.length > 0;
    const hasPaused = pausedIds.length > 0;

    const pauseAll = async () => {
        await Promise.allSettled(activeIds.map(id => invoke('pause_download', { downloadId: id })));
    };

    const resumeAll = async () => {
        await Promise.allSettled(pausedIds.map(id => invoke('resume_download', { downloadId: id })));
    };

    const TAB_CONFIG: Record<TabId, TopBarConfig> = {
        all: {
            buttons: [
                { label: 'Resume All', icon: '▶', onClick: resumeAll, disabled: !hasPaused },
                { label: 'Pause All', icon: '⏸', onClick: pauseAll, disabled: !hasActive },
                { label: 'Clear Finished', icon: '🗑', onClick: clearCompleted },
            ],
        },
        downloading: {
            buttons: [
                { label: 'Pause All', icon: '⏸', onClick: pauseAll, disabled: !hasActive },
                { label: 'Speed Limiter', icon: '⚡', onClick: () => { } },
                { label: 'Global Stats', icon: '📊', onClick: () => { } },
            ],
        },
        paused: {
            buttons: [
                { label: 'Resume All', icon: '▶', onClick: resumeAll, disabled: !hasPaused },
                { label: 'Force Start', icon: '🚀', onClick: resumeAll, disabled: !hasPaused },
                { label: 'Clear Queue', icon: '🗑', onClick: clearCompleted },
            ],
        },
        completed: {
            buttons: [
                { label: 'Clear History', icon: '🗑', onClick: clearHistory },
            ],
        },
        settings: {
            buttons: [],
        },
    };

    const config = TAB_CONFIG[currentTab];

    return (
        <header className="topbar">
            {config.buttons.map((btn) => (
                <button
                    key={btn.label}
                    className="topbar-btn"
                    onClick={btn.onClick}
                    disabled={btn.disabled}
                    title={btn.label}
                >
                    <span>{btn.icon}</span>
                    <span>{btn.label}</span>
                </button>
            ))}

            <div className="topbar-spacer" />

            <div className="topbar-search">
                <span className="topbar-search-icon">🔍</span>
                <input
                    type="text"
                    placeholder="Search downloads..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                />
            </div>
        </header>
    );
};
