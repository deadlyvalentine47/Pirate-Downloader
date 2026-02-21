// Background Service Worker

// ID of the native host (must match registry key)
const HOST_NAME = "com.piratedownloader.host";

// Connection to the native host
let port = null;

// Track if we are waiting for a link refresh
let isWaitingForRefresh = false;

// Store detected media with more metadata
let detectedMedia = [];
const MAX_MEDIA_ITEMS = 50;
const requestMetadata = new Map(); // url -> headers

function addMedia(url, type, tabTitle) {
    // Avoid duplicates
    if (detectedMedia.some(m => m.url === url)) return;
    
    // Use Tab Title if available, otherwise clean the URL
    let filename = tabTitle || url.split('/').pop().split('?')[0] || "stream";
    
    // Remove common annoying suffixes and clean up
    filename = filename.replace(/\.(m3u8|mpd|ts|mp4|zip|exe)$/i, '')
                       .replace(/[_-]/g, ' ')
                       .trim();

    // If filename is still a mess (like a hash or too short), fallback to "Video Stream"
    if (filename.length < 3 || /^[a-zA-Z0-9]{10,}$/.test(filename)) {
        filename = "Video Stream " + (detectedMedia.length + 1);
    }
    
    detectedMedia.unshift({
        url: url,
        type: type,
        filename: filename,
        timestamp: Date.now()
    });

    // Limit size
    if (detectedMedia.length > MAX_MEDIA_ITEMS) {
        detectedMedia.pop();
    }
}

function connectToHost() {
    console.log("Connecting to native host:", HOST_NAME);
    port = chrome.runtime.connectNative(HOST_NAME);
    
    port.onMessage.addListener((msg) => {
        console.log("Received message from host:", msg);
        if (msg.type === "WAIT_FOR_LINK") {
            isWaitingForRefresh = true;
            chrome.notifications.create({
                type: "basic",
                iconUrl: "icons/icon128.png",
                title: "Link Expired 🏴‍☠️",
                message: "Please visit the download page to refresh the link."
            });
        }
    });
    
    port.onDisconnect.addListener(() => {
        console.log("Disconnected from host", chrome.runtime.lastError);
        port = null;
    });
}

connectToHost();

// Listen for messages from popup
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
    if (request.type === "PING_HOST") {
        if (!port) connectToHost();
        port.postMessage({ type: "ECHO", payload: "PING from Popup!" });
        sendResponse({ status: "Message sent to host" });
    } else if (request.type === "GET_MEDIA") {
        sendResponse({ media: detectedMedia });
    } else if (request.type === "CLEAR_MEDIA") {
        detectedMedia = [];
        sendResponse({ success: true });
    } else if (request.type === "DOWNLOAD_MEDIA") {
        // IDM SECRET: Fetch LATEST headers for this URL at the moment of download
        const headers = requestMetadata.get(request.url) || {};
        
        sendToHost("DOWNLOAD_REQUEST", {
            url: request.url,
            filename: request.filename,
            referrer: request.referrer || sender.url || "",
            headers: headers
        });
        sendResponse({ success: true });
    }
    return true;
});

// Helper to send typed messages to Host
function sendToHost(type, payload) {
    if (!port) connectToHost();
    console.log("Sending to host:", type, payload);
    port.postMessage({ type: type, payload: payload });
}

// Capture Headers (The IDM DNA)
chrome.webRequest.onBeforeSendHeaders.addListener(
    (details) => {
        const headers = {};
        const skip = ['host', 'connection', 'content-length', 'proxy-authorization'];
        
        for (const header of details.requestHeaders) {
            const name = header.name.toLowerCase();
            if (!skip.includes(name)) {
                headers[header.name] = header.value;
            }
        }
        
        requestMetadata.set(details.url, headers);
        setTimeout(() => requestMetadata.delete(details.url), 600000);
    },
    { urls: ["<all_urls>"] },
    ["requestHeaders", "extraHeaders"]
);

// Sniffer
chrome.webRequest.onBeforeRequest.addListener(
    (details) => {
        if (details.method !== "GET") return;
        const url = details.url.split('?')[0];

        if (url.endsWith(".m3u8") || url.endsWith(".mpd")) {
            console.log("Media stream detected:", details.url);
            if (details.tabId >= 0) {
                chrome.tabs.get(details.tabId, (tab) => {
                    const title = tab ? tab.title : null;
                    addMedia(details.url, url.endsWith(".m3u8") ? "HLS" : "DASH", title);
                });
            } else {
                addMedia(details.url, url.endsWith(".m3u8") ? "HLS" : "DASH", null);
            }
        }
    },
    { urls: ["<all_urls>"] }
);

// Context Menus
chrome.runtime.onInstalled.addListener(() => {
    chrome.contextMenus.create({ id: "download-link", title: "Download Link with Pirate", contexts: ["link"] });
    chrome.contextMenus.create({ id: "download-media", title: "Download Media with Pirate", contexts: ["video", "audio"] });
});

chrome.contextMenus.onClicked.addListener((info, tab) => {
    let url = info.menuItemId === "download-link" ? info.linkUrl : info.srcUrl;
    if (url) {
        const headers = requestMetadata.get(url) || {};
        sendToHost("DOWNLOAD_REQUEST", {
            url: url,
            headers: headers,
            referrer: info.pageUrl
        });
    }
});
