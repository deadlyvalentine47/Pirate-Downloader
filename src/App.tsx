import { useEffect } from "react";
import { useDownloadStore } from "./stores/downloadStore";
import { useHistoryStore } from "./stores/historyStore";
import { useTauriEvents } from "./hooks/useTauriEvents";
import { DownloadControls } from "./components/download/DownloadControls";
import { DownloadStatus } from "./components/download/DownloadStatus";
import { ActionButtons } from "./components/download/ActionButtons";
import { ProgressBar } from "./components/common/ProgressBar";
import { HistoryList } from "./components/history/HistoryList";
import { IPCConfirmation } from "./components/download/IPCConfirmation";
import "./App.css";

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
    <main className="app-container">
      <h1>🏴‍☠️ Pirate Downloader</h1>

      {/* Download Controls */}
      <DownloadControls />

      {/* Progress Bar */}
      <ProgressBar progress={progress} totalSize={totalSize} />

      {/* Action Buttons (Pause/Resume/Stop/Cancel) */}
      {downloadId && downloadState !== 'idle' && (
        <div className="action-buttons-wrapper">
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

      {/* IPC Confirmation Modal */}
      <IPCConfirmation />
    </main>
  );
}

export default App;
