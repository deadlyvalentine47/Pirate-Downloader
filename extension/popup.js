// Pirate Sniffer Popup Logic

document.addEventListener('DOMContentLoaded', () => {
    const mediaList = document.getElementById('media-list');
    const mediaCount = document.getElementById('media-count');
    const clearBtn = document.getElementById('clear-all-btn');
    const pingBtn = document.getElementById('ping-btn');
    const status = document.getElementById('status');

    // 1. Fetch and render media
    const refreshMedia = () => {
        chrome.runtime.sendMessage({ type: "GET_MEDIA" }, (response) => {
            if (response && response.media) {
                renderMedia(response.media);
            }
        });
    };

    const renderMedia = (media) => {
        mediaCount.textContent = media.length;
        
        if (media.length === 0) {
            mediaList.innerHTML = '<div class="empty-state">No media detected yet. Sail the web to find treasure!</div>';
            return;
        }

        mediaList.innerHTML = '';
        media.forEach(item => {
            const div = document.createElement('div');
            div.className = 'media-item';
            div.innerHTML = `
                <div class="media-info">
                    <div class="media-name" title="${item.filename}">${item.filename}</div>
                    <div class="media-type">${item.type}</div>
                </div>
                <div class="media-url" title="${item.url}">${item.url}</div>
                <button class="download-btn" data-url="${item.url}" data-filename="${item.filename}">
                    Download with Pirate
                </button>
            `;
            mediaList.appendChild(div);
        });

        // Add event listeners to buttons
        document.querySelectorAll('.download-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const url = e.target.getAttribute('data-url');
                const filename = e.target.getAttribute('data-filename');
                downloadMedia(url, filename);
            });
        });
    };

    const downloadMedia = (url, filename) => {
        chrome.runtime.sendMessage({ 
            type: "DOWNLOAD_MEDIA", 
            url: url, 
            filename: filename 
        }, (response) => {
            status.textContent = "Sent to app!";
            setTimeout(() => status.textContent = "Ready", 2000);
        });
    };

    // 2. Clear functionality
    clearBtn.addEventListener('click', () => {
        chrome.runtime.sendMessage({ type: "CLEAR_MEDIA" }, (response) => {
            refreshMedia();
        });
    });

    // 3. Debug Ping
    pingBtn.addEventListener('click', () => {
        chrome.runtime.sendMessage({ type: "PING_HOST" }, (response) => {
            status.textContent = "Ping sent!";
            status.style.color = "#38bdf8";
            setTimeout(() => {
                status.textContent = "Ready";
                status.style.color = "#64748b";
            }, 2000);
        });
    });

    // Initial load
    refreshMedia();
});
