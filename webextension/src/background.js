function show_error(error) {
    // Calling alert from a background script isn't allowed. So this is a
    // workaround where the alert is injected into the active tab.
    // https://stackoverflow.com/a/41546651
    var alert_window = `alert("${error}")`;
    browser.tabs.executeScript({code: alert_window});
}

function get_values() {
    return Promise.all([
        browser.storage.sync.get("base_url").then(({base_url}) =>
            base_url),
        browser.storage.sync.get("access_token").then(({access_token}) =>
            access_token),
        browser.storage.sync.get("refresh_token").then(({refresh_token}) =>
            refresh_token),
    ]).then(values => ({base_url: values[0], access_token: values[1], refresh_token: values[2]}));
}

function check_values(values) {
    return new Promise((resolve, reject) => {
        for(let key of Object.keys(values)) {
            if(values[key] === undefined || values[key] === null) {
                reject(`${key} is ${values[key]}`);
            }
        }
        resolve(values);
    });
}

function send_request(base_url, access_token, url, content) {
    return new Promise((resolve, reject) => {
        try {
            const req = new XMLHttpRequest();
            req.open("POST", `${base_url}/api/fetch?auth-type=oauth2`);
            req.setRequestHeader("authorization", `bearer ${access_token}`);
            req.setRequestHeader("content-type", "application/json");
            req.onreadystatechange = () => {
                if(req.readyState === 4) {
                    if(req.status !== 200 && req.status !== 201) {
                        reject({"code": req.status, "message": `Error fetching ${url}: ${req.status} - ${req.statusText}`});
                    } else {
                        resolve();
                    }
                }
            };
            req.send(JSON.stringify({"url": url, "html": content}));
        } catch(error) {
            reject({"code": 0, "message": `Error fetching ${url}: ${error}`});
        }
    });
}

function send_refresh_token_request(base_url, refresh_token) {
    return new Promise((resolve, reject) => {
        try {
            const req = new XMLHttpRequest();
            req.open("POST", `${base_url}/api/auth/oauth2/refresh-token`);
            req.setRequestHeader("content-type", "application/json");
            req.responseType = "json";
            req.onreadystatechange = () => {
                if(req.readyState === 4) {
                    if(req.status !== 200) {
                        reject(`Error refreshing token: ${req.status} - ${req.statusText}`);
                    }
                }
            };
            req.send(JSON.stringify({"refresh_token": refresh_token}));
            req.addEventListener("load", e => {
                const {refresh_token, access_token} = e.target.response;
                if(refresh_token === undefined || refresh_token === null ||
                        access_token === undefined || access_token === null) {
                    console.error("Invalid response for refresh token request", e.target.response);
                    reject("Unable to refresh access token. Please refresh the token from the addon options.");
                }
                browser.storage.sync.set({refresh_token, access_token});
                resolve({refresh_token, access_token});
            });
        } catch(error) {
            reject(`Error refreshing token: ${error}. Please refresh the token from the addon options.`);
        }
    });
}

const request_fetch_page = ({content, url}) => {
    get_values()
        .then(check_values)
        .then(({base_url, access_token, refresh_token}) => {
            send_request(base_url, access_token, url, content)
                .catch(error => {
                    console.error(error);
                    // Only try refreshing the access token if the error was access denied
                    if(error.code === 401) {
                        send_refresh_token_request(base_url, refresh_token)
                            .then(({access_token, refresh_token}) => send_request(base_url, access_token, url, content))
                            .catch(error => show_error(error.message));
                    } else {
                        show_error(error.message);
                    }
                })
        })
        .catch(error => {
            show_error(`Error fetching values from storage: ${error}. Please setup the addon from the addon options.`);
        });
}

const send_message_to_content_script = tab => {
    browser.tabs.sendMessage(tab.id, null);
}

// When the button or the keyboard shortcut is pressed the background script
// sends a message to the content script of the current tab which then serializes
// the contents of the page and sends it back here.
browser.browserAction.onClicked.addListener(send_message_to_content_script);
browser.commands.onCommand.addListener(command => {
    if(command == "fetch-page") {
        // tabs.getCurrent returns undefined when run from a background script so that cannot be used here.
        // https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/tabs/getCurrent
        const tabs = browser.tabs.query({"active": true, "currentWindow": true});
        tabs.then(tabs => {
            if(tabs.length !== 1) {
                show_error("Error getting the current tab");
            } else {
                send_message_to_content_script(tabs[0]);
            }
        });
    }
})
browser.runtime.onMessage.addListener(request_fetch_page);
