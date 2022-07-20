function save_values(values) {
    browser.storage.sync.set(values);
}

function display_access_token(access_token) {
    const access_token_text = `${access_token.substring(0, 5)}...${access_token.substring(access_token.length - 5)}`;
    document.querySelector("#access-token-header").style.display = "block";
    document.querySelector("#access-token").textContent = access_token_text;
}

function display_refresh_token(refresh_token) {
    const refresh_token_text = `${refresh_token.substring(0, 5)}...${refresh_token.substring(refresh_token.length - 5)}`;
    document.querySelector("#refresh-token-header").style.display = "block";
    document.querySelector("#refresh-token").textContent = refresh_token_text;
}

function set_access_token(data) {
    try {
        const url = new URL(data);
        const params = url.searchParams;
        if(!params.has("access_token")) {
            alert("No access token in received query params");
            return;
        }
        const access_token = params.get("access_token");
        const refresh_token = params.get("refresh_token");
        display_access_token(access_token);
        display_refresh_token(refresh_token);
        const base_url = document.querySelector("#base-url").value;
        save_values({base_url, access_token, refresh_token});
    } catch(error) {
        alert(`Error extracting access token: ${error}`);
    }
}

function launch_flow(e) {
    e.preventDefault();
    const base_url = document.querySelector("#base-url").value;

    // Instead of requesting host permissions for all urls at install time,
    // we request permission for the base url here.
    // https://stackoverflow.com/a/71913707
    // This must be called from a function which responds to user input.
    try {
        browser.permissions.request({
            permissions: ["webRequest"],
            origins: [`${base_url}/*`]
        });
    } catch(error) {
        alert(`Error requesting permissions for ${base_url}/*`);
    }

    const input_map = {"base url": base_url};
    for(let name in input_map) {
        const v = input_map[name];
        if(v === "" || v === undefined || v === null) {
            alert(`You must set the ${name}`);
            return;
        }
    }
    const app_host = navigator.userAgent;
    const redirect_uri = browser.identity.getRedirectURL();
    const url = `${base_url}/auth/oauth2/authorize?redirect_uri=${encodeURIComponent(redirect_uri)}&app_host=${encodeURIComponent(app_host)}`;
    // https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/identity/launchWebAuthFlow
    browser.identity.launchWebAuthFlow({
        "interactive": true,
        "url": url
    })
        .then(set_access_token)
        .catch(error => alert(`Error when authorizing: ${error}`));
}

function restore_values() {
    browser.storage.sync.get("access_token").then(({access_token}) =>
        display_access_token(access_token));
    browser.storage.sync.get("refresh_token").then(({refresh_token}) =>
        display_refresh_token(refresh_token));
    browser.storage.sync.get("base_url").then(({base_url}) =>
        document.querySelector("#base-url").value = base_url
    );
}

document.addEventListener("DOMContentLoaded", restore_values);
document.querySelector("form").addEventListener("submit", launch_flow);
