// History state management with Zustand
import { create } from 'zustand';
import type { HistoryItem } from '../types';
import { loadHistory, addHistoryItem, clearHistory as clearHistoryStorage } from '../utils/storage';

interface HistoryStore {
    history: HistoryItem[];

    // Actions
    loadHistory: () => void;
    addItem: (url: string, path: string, size: number, status: 'Success' | 'Failed') => void;
    clearHistory: () => void;
}

export const useHistoryStore = create<HistoryStore>((set) => ({
    history: [],

    loadHistory: () => {
        const history = loadHistory();
        set({ history });
    },

    addItem: (url, path, size, status) => {
        const newHistory = addHistoryItem(url, path, size, status);
        set({ history: newHistory });
    },

    clearHistory: () => {
        clearHistoryStorage();
        set({ history: [] });
    },
}));
