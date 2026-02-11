// Download Controls Component
import { useDownloadStore } from '../../../stores/downloadStore';
import { useDownload } from '../../../hooks/useDownload';
import './style.css';

export const DownloadControls = () => {
    const { url, savePath, threads, setUrl, setThreads } = useDownloadStore();
    const { browseFile, startDownload } = useDownload();

    return (
        <div className="download-controls-container">
            <div className="control-group">
                <label className="control-label">URL:</label>
                <input
                    className="control-input"
                    value={url}
                    onChange={(e) => setUrl(e.target.value)}
                    placeholder="https://..."
                />
            </div>

            <div className="file-input-group">
                <input
                    className="file-path-input"
                    value={savePath}
                    placeholder="Save location..."
                    readOnly
                />
                <button onClick={browseFile} className="browse-btn">
                    📂 Browse...
                </button>
            </div>

            <div className="control-group">
                <label>⚡ Threads: {threads}</label>
                <input
                    type="range"
                    min="1"
                    max="32"
                    value={threads}
                    onChange={(e) => setThreads(Number(e.target.value))}
                    className="threads-input"
                />
            </div>

            <button
                onClick={startDownload}
                className="start-download-btn"
            >
                Start Download 🚀
            </button>
        </div>
    );
};
