// Background Service Worker

// ID of the native host (must match registry key)
const HOST_NAME = "com.piratedownloader.host";

// Connection to the native host
let port = null;

function connectToHost() {
    console.log("Connecting to native host:", HOST_NAME);
    port = chrome.runtime.connectNative(HOST_NAME);

    port.onMessage.addListener((msg) => {
        console.log("Received message from host:", msg);
    });

    port.onDisconnect.addListener(() => {
        console.log("Disconnected from host", chrome.runtime.lastError);
        port = null;
    });
}

// Connect on startup
connectToHost();

// Test function to send echo
function sendEcho(text) {
    if (!port) connectToHost();
    console.log("Sending echo:", text);
    port.postMessage({ type: "ECHO", payload: text });
}

// Expose for debugging via console
self.sendEcho = sendEcho;

// Listen for messages from popup
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
    if (request.type === "PING_HOST") {
        sendEcho("PING from Popup!");
        sendResponse({ status: "Message sent to host" });
    }
});

// Listen for extension icon click (if no popup) or other events
// Listen for extension icon click (if no popup) or other events
chrome.runtime.onInstalled.addListener(() => {
    console.log("Pirate Downloader Extension Installed");

    // Create Context Menu Items
    chrome.contextMenus.create({
        id: "download-link",
        title: "Download Link with Pirate",
        contexts: ["link"]
    });

    chrome.contextMenus.create({
        id: "download-media",
        title: "Download Media with Pirate",
        contexts: ["image", "video", "audio"]
    });

    chrome.contextMenus.create({
        id: "download-page",
        title: "Download Page with Pirate",
        contexts: ["page"]
    });
});

// Handle Context Menu Clicks
chrome.contextMenus.onClicked.addListener((info, tab) => {
    let url = "";
    let filename = undefined;

    if (info.menuItemId === "download-link") {
        url = info.linkUrl;
    } else if (info.menuItemId === "download-media") {
        url = info.srcUrl;
    } else if (info.menuItemId === "download-page") {
        url = info.pageUrl;
    }

    if (url) {
        console.log("Context Menu clicked:", info.menuItemId, url);

        // Construct payload matching pirate-shared::DownloadRequest
        const payload = {
            url: url,
            filename: filename, // Let backend/host determine or we can sniff
            headers: {}, // TODO: Extract headers if possible via webRequest
            referrer: info.pageUrl
        };

        sendToHost("DOWNLOAD_REQUEST", payload);
    }
});

// Helper to send typed messages to Host
function sendToHost(type, payload) {
    if (!port) connectToHost();
    console.log("Sending to host:", type, payload);
    port.postMessage({ type: type, payload: payload });
}

// Intercept Downloads
chrome.downloads.onCreated.addListener((downloadItem) => {
    // specific checking to avoid loops or unwanted interceptions could go here
    // For now, we intercept eveything that has a valid URL
    if (downloadItem.state !== "in_progress" && downloadItem.state !== "interrupted") {
        return;
    }

    console.log("Intercepting download:", downloadItem);

    // Cancel the browser download immediately
    chrome.downloads.cancel(downloadItem.id, () => {
        if (chrome.runtime.lastError) {
            console.error("Failed to cancel download:", chrome.runtime.lastError);
        } else {
            console.log("Browser download cancelled. Offloading to Pirate.");

            // Send to Native Host
            const payload = {
                url: downloadItem.url,
                filename: downloadItem.filename, // Might be empty/provisional
                referrer: downloadItem.referrer
            };
            sendToHost("DOWNLOAD_REQUEST", payload);
        }
    });
});
