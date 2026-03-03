import { useDownloadStore } from '../../stores/downloadStore';
import { useUIStore, type TabId } from '../../stores/uiStore';

const NAV_ITEMS: { id: TabId; icon: string; label: string }[] = [
    { id: 'all', icon: '⬇', label: 'All Downloads' },
    { id: 'downloading', icon: '▶', label: 'Downloading' },
    { id: 'paused', icon: '⏸', label: 'Paused / Queued' },
    { id: 'completed', icon: '✓', label: 'Completed' },
];

export const Sidebar = () => {
    const { currentTab, setCurrentTab, setShowAddModal } = useUIStore();
    const { downloads } = useDownloadStore();

    const activeCount = downloads.filter(d => d.status === 'active').length;

    return (
        <aside className="sidebar">
            {/* Logo */}
            <div className="sidebar-logo">
                <span className="sidebar-logo-icon">🏴‍☠️</span>
                <span className="sidebar-logo-text">Pirate Downloader</span>
            </div>

            {/* Add New Download CTA */}
            <button
                className="sidebar-add-btn"
                onClick={() => setShowAddModal(true)}
            >
                <span>+</span>
                <span>Add New Download</span>
            </button>

            {/* Categories */}
            <div className="sidebar-section-label">Categories</div>
            <nav className="sidebar-nav">
                {NAV_ITEMS.map(({ id, icon, label }) => (
                    <div
                        key={id}
                        className={`sidebar-nav-item ${currentTab === id ? 'active' : ''}`}
                        onClick={() => setCurrentTab(id)}
                    >
                        <span className="sidebar-nav-icon">{icon}</span>
                        <span className="sidebar-nav-label">{label}</span>
                        {id === 'downloading' && activeCount > 0 && (
                            <span className="sidebar-badge">{activeCount}</span>
                        )}
                    </div>
                ))}
            </nav>

            <div className="sidebar-separator" />

            {/* System */}
            <div className="sidebar-section-label">System</div>
            <div className="sidebar-system">
                <div
                    className={`sidebar-nav-item ${currentTab === 'settings' ? 'active' : ''}`}
                    onClick={() => setCurrentTab('settings')}
                >
                    <span className="sidebar-nav-icon">⚙</span>
                    <span className="sidebar-nav-label">Settings</span>
                </div>
                <ExtensionStatus />
            </div>
        </aside>
    );
};

/* Extension status indicator — tries ping on mount */
const ExtensionStatus = () => {
    // We simply show connected status based on whether there are recent IPC messages.
    // For now it's a static indicator; in a future iteration it can ping the IPC server.
    return (
        <div className="sidebar-ext-status">
            <span className="sidebar-nav-icon">🧩</span>
            <span style={{ flex: 1, fontSize: '12px', color: 'var(--text-muted)' }}>Extension Status</span>
            <span className="sidebar-ext-dot connected" title="Connected" />
        </div>
    );
};
