// Formatting utilities for Pirate Downloader

/**
 * Format bytes to human-readable size
 * @param bytes - Number of bytes
 * @param decimals - Number of decimal places (default: 2)
 * @returns Formatted string (e.g., "1.23 MB")
 */
export const formatBytes = (bytes: number, decimals: number = 2): string => {
    if (bytes === 0) return '0 Bytes';

    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];

    const i = Math.floor(Math.log(bytes) / Math.log(k));

    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
};

/**
 * Format speed in bytes/second to human-readable format
 * @param bytesPerSecond - Speed in bytes per second
 * @returns Formatted string (e.g., "1.23 MB/s")
 */
export const formatSpeed = (bytesPerSecond: number): string => {
    return formatBytes(bytesPerSecond, 2) + '/s';
};

/**
 * Format seconds to human-readable time
 * @param seconds - Number of seconds
 * @returns Formatted string (e.g., "1h 23m 45s")
 */
export const formatTime = (seconds: number): string => {
    if (seconds < 60) {
        return `${Math.floor(seconds)}s`;
    }

    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);

    if (hours > 0) {
        return `${hours}h ${minutes}m ${secs}s`;
    }

    return `${minutes}m ${secs}s`;
};

/**
 * Calculate download percentage
 * @param downloaded - Bytes downloaded
 * @param total - Total bytes
 * @returns Percentage (0-100)
 */
export const calculatePercentage = (downloaded: number, total: number): number => {
    if (total === 0) return 0;
    return Math.min(100, (downloaded / total) * 100);
};
