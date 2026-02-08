// Download Controls Component
import { useDownloadStore } from '../../stores/downloadStore';
import { useDownload } from '../../hooks/useDownload';

export const DownloadControls = () => {
    const { url, savePath, threads, setUrl, setThreads } = useDownloadStore();
    const { browseFile, startDownload } = useDownload();

    return (
        <div style={{
            display: 'flex',
            flexDirection: 'column',
            gap: '15px',
            padding: '20px',
            border: '1px solid #ccc',
            borderRadius: '8px'
        }}>
            <div>
                <label style={{ fontWeight: 'bold' }}>URL:</label>
                <input
                    value={url}
                    onChange={(e) => setUrl(e.target.value)}
                    placeholder="https://..."
                    style={{ width: '100%', padding: '10px', marginTop: '5px' }}
                />
            </div>

            <div style={{ display: 'flex', gap: '10px' }}>
                <input
                    value={savePath}
                    placeholder="Save location..."
                    readOnly
                    style={{ flex: 1, padding: '10px', background: '#f0f0f0' }}
                />
                <button onClick={browseFile} style={{ padding: '10px 20px' }}>
                    📂 Browse...
                </button>
            </div>

            <div>
                <label>⚡ Threads: {threads}</label>
                <input
                    type="range"
                    min="1"
                    max="32"
                    value={threads}
                    onChange={(e) => setThreads(Number(e.target.value))}
                    style={{ width: '100%' }}
                />
            </div>

            <button
                onClick={startDownload}
                style={{
                    padding: '15px',
                    fontSize: '1.2em',
                    background: '#007acc',
                    color: 'white',
                    border: 'none',
                    borderRadius: '5px',
                    cursor: 'pointer'
                }}
            >
                Start Download 🚀
            </button>
        </div>
    );
};
