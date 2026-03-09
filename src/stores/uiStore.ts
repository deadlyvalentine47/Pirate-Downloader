// UI state management with Zustand
import { create } from 'zustand';
import { downloadDir } from '@tauri-apps/api/path';

export type TabId = 'all' | 'downloading' | 'paused' | 'completed' | 'settings';

interface UIStore {
    currentTab: TabId;
    showAddModal: boolean;
    searchQuery: string;
    defaultDownloadPath: string;        // persists across tab switches

    setCurrentTab: (tab: TabId) => void;
    setShowAddModal: (show: boolean) => void;
    setSearchQuery: (q: string) => void;
    setDefaultDownloadPath: (path: string) => void;
    initDefaultDownloadPath: () => Promise<void>;
}

export const useUIStore = create<UIStore>((set, get) => ({
    currentTab: 'all',
    showAddModal: false,
    searchQuery: '',
    defaultDownloadPath: '',

    setCurrentTab: (tab) => set({ currentTab: tab }),
    setShowAddModal: (show) => set({ showAddModal: show }),
    setSearchQuery: (q) => set({ searchQuery: q }),
    setDefaultDownloadPath: (path) => set({ defaultDownloadPath: path }),

    // Call once on app mount — resolves the system Downloads folder and stores it.
    // No-op if already set (i.e. user changed it).
    initDefaultDownloadPath: async () => {
        if (get().defaultDownloadPath) return;
        try {
            const dir = await downloadDir();
            set({ defaultDownloadPath: dir });
        } catch { }
    },
}));
