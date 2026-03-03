import { useUIStore } from '../../stores/uiStore';
import { DownloadTable } from '../download/DownloadTable';
import { SettingsView } from '../settings/SettingsView';

const TAB_META: Record<string, { title: string; icon: string }> = {
    all: { title: 'All Downloads', icon: '⬇' },
    downloading: { title: 'Downloading', icon: '▶' },
    paused: { title: 'Paused / Queued', icon: '⏸' },
    completed: { title: 'Completed', icon: '✓' },
    settings: { title: 'Settings', icon: '⚙' },
};

export const MainContent = () => {
    const { currentTab } = useUIStore();
    const meta = TAB_META[currentTab];

    return (
        <main className="main-content">
            <div className="main-content-header">
                <h1 className="main-content-title">{meta.title}</h1>
            </div>
            <div className="main-content-body">
                {currentTab === 'settings' ? (
                    <SettingsView />
                ) : (
                    <DownloadTable filter={currentTab} />
                )}
            </div>
        </main>
    );
};
