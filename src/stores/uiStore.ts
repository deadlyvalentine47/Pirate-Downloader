// UI state management with Zustand
import { create } from 'zustand';

export type TabId = 'all' | 'downloading' | 'paused' | 'completed' | 'settings';

interface UIStore {
    currentTab: TabId;
    showAddModal: boolean;
    searchQuery: string;

    setCurrentTab: (tab: TabId) => void;
    setShowAddModal: (show: boolean) => void;
    setSearchQuery: (q: string) => void;
}

export const useUIStore = create<UIStore>((set) => ({
    currentTab: 'all',
    showAddModal: false,
    searchQuery: '',

    setCurrentTab: (tab) => set({ currentTab: tab }),
    setShowAddModal: (show) => set({ showAddModal: show }),
    setSearchQuery: (q) => set({ searchQuery: q }),
}));
