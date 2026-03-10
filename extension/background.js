// Background Service Worker

// ID of the native host (must match registry key)
const HOST_NAME = "com.piratedownloader.host";

// Connection to the native host
let port = null;

// Track if we are waiting for a link refresh
let isWaitingForRefresh = false;

// Store detected media
let detectedMedia = [];
const MAX_MEDIA_ITEMS = 50;

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
            // Show notification to user
            chrome.notifications.create({
                type: "basic",
                iconUrl: "icons/icon128.png",
                title: "Link Expired 🏴‍☠️",
                message: "Please visit the download page to refresh the link."
            });
        }
    });
    
    // Update Native Host message handler (in host/src/main.rs we need to handle this)
    // For now, assume the host just passes everything to extension.
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
    } else if (request.type === "GET_MEDIA") {
        sendResponse({ media: detectedMedia });
    } else if (request.type === "CLEAR_MEDIA") {
        detectedMedia = [];
        sendResponse({ success: true });
    } else if (request.type === "DOWNLOAD_MEDIA") {
        dispatchDownloadRequest("DOWNLOAD_REQUEST", request.url, request.filename, request.referrer || sender.url || "");
        sendResponse({ success: true });
    }
    return true; // Keep channel open for async responses
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
        dispatchDownloadRequest("DOWNLOAD_REQUEST", url, filename, info.pageUrl);
    }
});

// Helper to extract cookies, userAgent, and send to Native Host
function dispatchDownloadRequest(type, url, filename, referrer) {
    if (!url) return;

    const userAgent = navigator.userAgent;
    
    // Parse basis URL for cookies
    try {
        const parsedUrl = new URL(url);
        chrome.cookies.getAll({ url: parsedUrl.origin }, (cookies) => {
            let cookieStr = "";
            if (cookies && cookies.length > 0) {
                cookieStr = cookies.map(c => `${c.name}=${c.value}`).join('; ');
            }
            
            const payload = {
                url: url,
                filename: filename || undefined,
                referrer: referrer || "",
                cookies: cookieStr,
                user_agent: userAgent
            };
            sendToHost(type, payload);
        });
    } catch (e) {
        console.error("Invalid URL:", url, e);
        // Fallback without cookies
        const payload = {
            url: url,
            filename: filename || undefined,
            referrer: referrer || "",
            cookies: "",
            user_agent: userAgent
        };
        sendToHost(type, payload);
    }
}

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

            if (isWaitingForRefresh) {
                console.log("Captured refreshed link:", downloadItem.url);
                dispatchDownloadRequest("LINK_UPDATE", downloadItem.url, undefined, downloadItem.referrer);
                isWaitingForRefresh = false;
            } else {
                dispatchDownloadRequest("DOWNLOAD_REQUEST", downloadItem.url, downloadItem.filename, downloadItem.referrer);
            }
        }
    });
});

// Media Sniffer (HLS/DASH) + Link Refresh
chrome.webRequest.onBeforeRequest.addListener(
    (details) => {
        if (details.method !== "GET") return;
        
        const url = details.url.split('?')[0];

        // Link Refresh Logic (IDM Mode fallback for streams)
        // Note: Standard downloads are handled by chrome.downloads.onCreated above.
        // This is only necessary for video streams (HLS/DASH) that don't trigger downloads.
        if (isWaitingForRefresh) {
            if (url.endsWith(".m3u8") || url.endsWith(".mpd")) {
                console.log("Captured refreshed STREAM link:", details.url);
                dispatchDownloadRequest("LINK_UPDATE", details.url, undefined, details.initiator || details.documentUrl);
                isWaitingForRefresh = false;
            }
        }
        
        if (url.endsWith(".m3u8") || url.endsWith(".mpd")) {
            console.log("Media stream detected:", details.url);
            
            // Try to get tab title for a better filename
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
