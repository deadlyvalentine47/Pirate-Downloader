import { useEffect } from "react";
import { useHistoryStore } from "./stores/historyStore";
import { useTauriEvents } from "./hooks/useTauriEvents";
import { Sidebar } from "./components/layout/Sidebar";
import { TopBar } from "./components/layout/TopBar";
import { MainContent } from "./components/layout/MainContent";
import { AddDownloadModal } from "./components/download/AddDownloadModal";
import "./App.css";

function App() {
  const { loadHistory } = useHistoryStore();

  // Setup Tauri event listeners
  useTauriEvents();

  // Load history on mount
  useEffect(() => {
    loadHistory();
  }, [loadHistory]);

  return (
    <div className="app-layout">
      <Sidebar />
      <TopBar />
      <MainContent />
      <AddDownloadModal />
    </div>
  );
}

export default App;
