function send_page_content() {
    // https://stackoverflow.com/a/49472387
    const t = new XMLSerializer().serializeToString(document);
    browser.runtime.sendMessage({"content": t, "url": window.location.href});
}

try {
    browser.runtime.onMessage.addListener(send_page_content);
} catch(error) {
    console.error("Error sending message to background script", error);
    alert("Error sending message to background script");
}
