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
chrome.runtime.onInstalled.addListener(() => {
    console.log("Pirate Downloader Extension Installed");
});
