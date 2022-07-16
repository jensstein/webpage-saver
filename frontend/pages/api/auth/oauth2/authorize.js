import axios from "axios";
import crypto from "crypto";

import { set_cookie, get_jwt } from "../../../../helpers/cookies.js";
import { get_userinfo } from "../../../../helpers/userinfo.js";

// Some of the code here comes from this page:
// https://auth0.com/docs/get-started/authentication-and-authorization-flow/call-your-api-using-the-authorization-code-flow-with-pkce#authorize-user

function base64URLEncode(str) {
    return str.toString("base64")
        .replace(/\+/g, "-")
        .replace(/\//g, "_")
        .replace(/=/g, "");
}

function sha256(buffer) {
    return crypto.createHash("sha256").update(buffer).digest();
}

export default async function handler(req, res) {
    const _url = new URL(req.url, "http://localhost");
    const params = _url.searchParams;
    const { redirect_uri, app_host } = req.query;
    const jwt = get_jwt({req});
    if(jwt === null || jwt === undefined) {
        console.log("Missing jwt");
        return res.status(401).send();
    }
    get_userinfo(jwt).then(async userinfo => {
        const { username } = userinfo;
        const state = base64URLEncode(crypto.randomBytes(32));
        const verifier = base64URLEncode(crypto.randomBytes(32));
        const client_id = process.env.OAUTH2_CLIENT_ID;
        const auth_data = {
            state,
            redirect_uri,
            verifier,
            client_id,
            app_host,
        }
        // Set a very short-lived cookie with some data to transfer to the callback call.
        set_cookie({res}, `${username}-auth-data`, JSON.stringify(auth_data), 30);
        const base_url = process.env.OAUTH2_PROVIDER_BASE_URL;
        const challenge = base64URLEncode(sha256(verifier));
        const own_redirect_uri = `${process.env.SERVER_BASE_URL}/auth/oauth2/callback`
        // TODO: I'm not sure what the best method of storing this value is.
        const audience = process.env.OAUTH2_AUDIENCE;
        // Include the `offline_access` scope to get a refresh token:
        // https://auth0.com/docs/secure/tokens/refresh-tokens/get-refresh-tokens
        const url = `${base_url}/authorize?response_type=code&client_id=${client_id}&code_challenge=${challenge}&code_challenge_method=S256&redirect_uri=${own_redirect_uri}&scope=offline_access&audience=${audience}&state=${state}`
        res.redirect(302, url);
    }).catch(error => {
        console.log(`Error when getting userinfo: ${error}`);
        res.status(401).send();
    });
}
