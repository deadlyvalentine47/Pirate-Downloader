import { useEffect } from "react";
import { useDownloadStore } from "./stores/downloadStore";
import { useHistoryStore } from "./stores/historyStore";
import { useTauriEvents } from "./hooks/useTauriEvents";
import { DownloadControls } from "./components/download/DownloadControls";
import { DownloadStatus } from "./components/download/DownloadStatus";
import { ActionButtons } from "./components/download/ActionButtons";
import { ProgressBar } from "./components/common/ProgressBar";
import { HistoryList } from "./components/history/HistoryList";

function App() {
  // Get state from stores
  const { progress, totalSize, status, downloadId, downloadState, setDownloadState } = useDownloadStore();
  const { history, loadHistory } = useHistoryStore();

  // Setup Tauri event listeners
  useTauriEvents();

  // Load history on mount
  useEffect(() => {
    loadHistory();
  }, [loadHistory]);

  const handleStateChange = (newState: string) => {
    setDownloadState(newState as any);
  };

  return (
    <main className="container" style={{ padding: "20px", fontFamily: "sans-serif", maxWidth: "800px", margin: "0 auto" }}>
      <h1>🏴‍☠️ Pirate Downloader</h1>

      {/* Download Controls */}
      <DownloadControls />

      {/* Progress Bar */}
      <ProgressBar progress={progress} totalSize={totalSize} />

      {/* Action Buttons (Pause/Resume/Stop/Cancel) */}
      {downloadId && downloadState !== 'idle' && (
        <div style={{ marginTop: '15px', textAlign: 'center' }}>
          <ActionButtons
            downloadId={downloadId}
            state={downloadState}
            onStateChange={handleStateChange}
          />
        </div>
      )}

      {/* Status Display */}
      <DownloadStatus status={status} progress={progress} totalSize={totalSize} state={downloadState} />

      {/* History */}
      <HistoryList history={history} />
    </main>
  );
}

export default App;
