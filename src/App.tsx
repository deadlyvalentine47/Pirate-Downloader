import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { save } from "@tauri-apps/plugin-dialog";

interface HistoryItem {
  id: number;
  url: String;
  filename: String;
  size: number;
  date: String;
  status: String;
}

function App() {
  const [url, setUrl] = useState("");
  const [savePath, setSavePath] = useState("");
  const [progress, setProgress] = useState(0);
  const [totalSize, setTotalSize] = useState(0);
  const [status, setStatus] = useState("Idle");
  const [threads, setThreads] = useState(16);
  const [history, setHistory] = useState<HistoryItem[]>([]);

  // Load History on Boot
  useEffect(() => {
    const saved = localStorage.getItem("download_history");
    if (saved) setHistory(JSON.parse(saved));
    const unlisten = listen<number>("download-progress", (event) => {
      setProgress(event.payload);
    });
    return () => { unlisten.then(f => f()); };
  }, []);

  // Save History Helper
  const addToHistory = (url: string, path: string, size: number, status: string) => {
    const newItem = {
      id: Date.now(),
      url: url.substring(0, 40) + "...",
      filename: path.split(/[\\/]/).pop() || "file",
      size: size,
      date: new Date().toLocaleTimeString(),
      status: status
    };
    const newHistory = [newItem, ...history];
    setHistory(newHistory);
    localStorage.setItem("download_history", JSON.stringify(newHistory));
  };

  async function browseFile() {
    // 1. Smart Detect Filename
    let defaultJavaName = "download.dat";
    let detectedSize = 0;
    if (url) {
      setStatus("Checking URL...");
      try {
        // Ask Rust for details
        const [name, size] = await invoke<[string, number]>("get_file_details", { url });
        defaultJavaName = name as string;
        detectedSize = size as number;
        setTotalSize(detectedSize);
        setStatus("Detected: " + name);
      } catch (e) {
        console.log("Could not detect filename", e);
      }
    }

    // 2. Open Dialog
    const path = await save({
      defaultPath: defaultJavaName,
      filters: [{ name: "All Files", extensions: ["*"] }]
    });
    if (path) {
      setSavePath(path);
      // If we didn't detect size earlier, we might want to try again or just wait start
      if (detectedSize > 0) setTotalSize(detectedSize);
      setStatus("Ready");
    }
  }

  async function startDownload() {
    if (!url || !savePath) {
      alert("Please select a URL and a Save Path!");
      return;
    }
    setStatus("Downloading...");
    try {
      await invoke("download_file", {
        url: url,
        filepath: savePath,
        threads: Number(threads)
      });
      setStatus("Finished");
      addToHistory(url, savePath, totalSize, "Success");
    } catch (e) {
      setStatus("Error: " + e);
      addToHistory(url, savePath, 0, "Failed");
    }
  }

  return (
    <main className="container" style={{ padding: "20px", fontFamily: "sans-serif", maxWidth: "800px", margin: "0 auto" }}>
      <h1>🏴‍☠️ Pirate Downloader</h1>

      {/* Controls */}
      <div style={{ display: "flex", flexDirection: "column", gap: "15px", padding: "20px", border: "1px solid #ccc", borderRadius: "8px" }}>
        <div>
          <label style={{ fontWeight: "bold" }}>URL:</label>
          <input
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder="https://..."
            style={{ width: "100%", padding: "10px", marginTop: "5px" }}
          />
        </div>
        <div style={{ display: "flex", gap: "10px" }}>
          <input
            value={savePath}
            placeholder="Save location..."
            readOnly
            style={{ flex: 1, padding: "10px", background: "#f0f0f0" }}
          />
          <button onClick={browseFile} style={{ padding: "10px 20px" }}>📂 Browse...</button>
        </div>
        <div>
          <label>⚡ Threads: {threads}</label>
          <input
            type="range" min="1" max="32"
            value={threads}
            onChange={(e) => setThreads(Number(e.target.value))}
            style={{ width: "100%" }}
          />
        </div>
        <button onClick={startDownload} style={{ padding: "15px", fontSize: "1.2em", background: "#007acc", color: "white", border: "none", borderRadius: "5px", cursor: "pointer" }}>
          Start Download 🚀
        </button>
        {/* Progress Bar */}
        {(progress > 0) && (
          <div style={{ background: "#e0e0e0", borderRadius: "10px", height: "20px", overflow: "hidden" }}>
            <div style={{
              width: `${Math.min(100, (progress / Math.max(1, totalSize)) * 100)}%`,
              background: "#4caf50",
              height: "100%",
              transition: "width 0.2s"
            }}></div>
          </div>
        )}
        <div style={{ textAlign: "center" }}>
          <strong>{status}</strong>
          {totalSize > 0 && ` (${(progress / 1024 / 1024).toFixed(1)} / ${(totalSize / 1024 / 1024).toFixed(1)} MB)`}
        </div>
      </div>
      {/* History */}
      <h2 style={{ marginTop: "30px" }}>📜 History</h2>
      <div style={{ borderTop: "2px solid #eee" }}>
        {history.map((item) => (
          <div key={item.id} style={{ padding: "10px", borderBottom: "1px solid #eee", display: "flex", justifyContent: "space-between" }}>
            <div>
              <div style={{ fontWeight: "bold" }}>{item.filename}</div>
              <div style={{ fontSize: "0.8em", color: "#666" }}>{item.url}</div>
            </div>
            <div style={{ textAlign: "right" }}>
              <div style={{ color: item.status === "Success" ? "green" : "red" }}>{item.status}</div>
              <div style={{ fontSize: "0.8em", color: "#999" }}>{item.size > 0 ? (item.size / 1024 / 1024).toFixed(1) + " MB" : ""}</div>
            </div>
          </div>
        ))}
        {history.length === 0 && <p style={{ color: "#888", textAlign: "center", padding: "20px" }}>No downloads yet.</p>}
      </div>
    </main>
  );
}

export default App;