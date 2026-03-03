# Pirate Downloader - UI Architecture Redesign (v2.0)

To evolve from a simple single-page tool into a professional, modern desktop download manager, we will transition to a **Sidebar Navigation Layout** with a **Contextual Action Bar**. This design pattern is the industry standard for productivity apps, ensuring the UI only shows relevant tools at the right time.

---

## 1. Core Layout Structure

The app is divided into three persistent regions:

### A. The Sidebar (Left - Fixed Width)
The main navigation hub for filtering data and accessing system configurations.
* **Action Button:** A large, prominent **"+ Add New Download"** button at the top.
* **Categories:**
    * ⬇️ **All Downloads** (Global view of all tasks)
    * ▶️ **Downloading** (Active tasks only)
    * ⏸️ **Paused / Queued** (Tasks waiting or stopped)
    * ✅ **Completed** (History/Finished tasks)
* **System:**
    * ⚙️ **Settings** (Global preferences)
    * 🧩 **Extension Status** (Visual indicator: Green = Connected, Red = Disconnected)

### B. The Top Action Bar (Top - Fixed Height)
**Contextual Region:** The buttons here update dynamically based on the Sidebar selection or item selection.

| If Sidebar is... | Top Bar shows... |
| :--- | :--- |
| **All Downloads** | [Resume All] [Pause All] [Clear Finished] |
| **Downloading** | [Pause All] [Speed Limiter] [Global Stats] |
| **Paused / Queued** | [Resume All] [Force Start] [Clear Queue] |
| **Completed** | [Open Folder] [Clear History] [Redownload All] |

* **Search/Filter:** A permanent small search bar on the right side of the bar to filter the current view by filename.

### C. The Main Content Area (Center - Fluid)
Displays the content for the selected Sidebar category.
* **Downloads List View (Table):**
    * **File Icon:** Visual cue (e.g., Folder for zips, Play for video).
    * **File Name:** Primary identifier.
    * **Progress Bar:** Smooth, animated (Teal/Neon Blue).
    * **Metrics:** Speed (MB/s), ETA, and Size.
    * **Context Menu (Right-click):** Open Folder, Copy URL, Delete, Move to Top.
* **Settings View:** * **General:** Download paths, start on boot.
    * **Network:** Max parallel connections, thread count per download.
    * **Advanced:** YouTube/HLS stream engine toggles.

---

## 2. The "Add Download" Flow (Modal)
Clicking the **"+ Add New Download"** button triggers a sleek, centered Modal:
* **Smart Input:** URL text box that attempts to auto-paste from the clipboard.
* **Quick Config:** Save path selector, filename override, and thread slider (1-32).
* **Actions:** [Download Now] (High Priority) or [Add to Queue].

---

## 3. Visual Aesthetic & UX
* **Theme:** Modern "Dark Mode." Background: `#0f172a` (Slate-900). Sidebar: `#1e293b` (Slate-800).
* **Accent Colors:** **Pirate Gold** (`#fbbf24`) for warnings/pauses, **Neon Blue** (`#38bdf8`) for active progress.
* **States:** * **Disabled:** Actions not applicable to the current view are grayed out (e.g., "Pause All" is disabled in the "Completed" tab).
    * **Hover:** Subtle scale-up or highlight on list items.

---

## 4. Implementation Plan

1.  **Layout Scaffold:** Build the three-panel grid using Tailwind's `flex` or `grid`.
2.  **Contextual Logic:** * Create a `currentTab` state in React.
    * Map the `TopBar` buttons to the `currentTab` value using a configuration object.
3.  **The "Bridge":** * Connect the "Add Download" modal to the Rust `tauri::command` for fetching file metadata (size/name) before starting.
    * Use `listen` from `@tauri-apps/api/event` to update the progress bars in real-time.
4.  **Settings Storage:** Sync the Settings View with a local JSON config via the Rust backend.