document.getElementById('ping-btn').addEventListener('click', () => {
    // Send message to background script to trigger ping
    chrome.runtime.sendMessage({ type: "PING_HOST" }, (response) => {
        const status = document.getElementById('status');
        status.textContent = "Ping sent! Check background console.";
        status.style.color = "blue";
    });
});
