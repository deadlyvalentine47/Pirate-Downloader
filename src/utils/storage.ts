// LocalStorage utilities for Pirate Downloader
import type { HistoryItem } from '../types';

const HISTORY_KEY = 'download_history';

/**
 * Save download history to localStorage
 * @param history - Array of history items
 */
export const saveHistory = (history: HistoryItem[]): void => {
    try {
        localStorage.setItem(HISTORY_KEY, JSON.stringify(history));
    } catch (error) {
        console.error('Failed to save history:', error);
    }
};

/**
 * Load download history from localStorage
 * @returns Array of history items
 */
export const loadHistory = (): HistoryItem[] => {
    try {
        const saved = localStorage.getItem(HISTORY_KEY);
        if (saved) {
            return JSON.parse(saved);
        }
    } catch (error) {
        console.error('Failed to load history:', error);
    }
    return [];
};

/**
 * Add a new item to history
 * @param url - Download URL
 * @param path - Save path
 * @param size - File size in bytes
 * @param status - Download status
 * @returns Updated history array
 */
export const addHistoryItem = (
    url: string,
    path: string,
    size: number,
    status: 'Success' | 'Failed'
): HistoryItem[] => {
    const currentHistory = loadHistory();

    const newItem: HistoryItem = {
        id: Date.now(),
        url: url.substring(0, 40) + '...',
        filename: path.split(/[\\/]/).pop() || 'file',
        size: size,
        date: new Date().toLocaleTimeString(),
        status: status
    };

    const newHistory = [newItem, ...currentHistory];
    saveHistory(newHistory);

    return newHistory;
};

/**
 * Clear all history
 */
export const clearHistory = (): void => {
    try {
        localStorage.removeItem(HISTORY_KEY);
    } catch (error) {
        console.error('Failed to clear history:', error);
    }
};
