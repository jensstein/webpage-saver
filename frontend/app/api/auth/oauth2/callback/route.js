import axios from "axios";
import jsonwebtoken from "jsonwebtoken";

import { get_cookie, get_jwt } from "../../../../../helpers/cookies.js";
import { get_userinfo } from "../../../../../helpers/userinfo.js";

import { associate_app_to_user } from "../../../../../requests/associate-app-to-user.js";

// TODO: chrome support is currently untested.
const BROWSER_REDIRECTS = ["allizom.org", "chromiumapp.org", "dk.jens.webpagesaver://token-callback"];

// In order to transmit values from the first authorization call to this
// callback a short-lived cookie is set.
async function get_auth_data(cookies) {
    return new Promise((resolve, reject) => {
        const auth_data_cookie = cookies;
        if(auth_data_cookie === undefined || auth_data_cookie === null) {
            reject("Missing auth data");
        }
        const auth_data = JSON.parse(auth_data_cookie.value);
        resolve(auth_data);
    })
}

async function get_cookie_for_user(userinfo) {
    return new Promise((resolve, reject) => {
        const { username } = userinfo;
        const cookie_name = `${username}-auth-data`;
        get_cookie(cookie_name).then(cookie => {
            resolve(cookie)
        });
    });
}

function parse_redirect_uri(redirect_uri) {
    if(BROWSER_REDIRECTS.some(r => redirect_uri.includes(r))) {
        const redirect_uri_ = new URL(redirect_uri);
        const host = redirect_uri_.hostname;
        for(let r of BROWSER_REDIRECTS) {
            return host.substring(host.length - r.length);
        }
    }
    return redirect_uri;
}

// Here I validate that the cookie set during the first authorization call
// contains the expected values.
async function validate_auth_data(auth_data) {
    return new Promise((resolve, reject) => {
        for(const attribute of ["redirect_uri", "state", "verifier", "client_id", "app_host"]) {
            if(!(attribute in auth_data)) {
                reject(`Missing ${attribute} from auth data`);
            }
        }
        try {
            let redirect_uri = parse_redirect_uri(auth_data["redirect_uri"]);
        } catch (error) {
            reject(`Unable to parse redirect_uri ${auth_data["redirect_uri"]}: ${error}`);
        }
        resolve(auth_data);
    })
}

function get_code(url) {
    return async auth_data => {
        return new Promise((resolve, reject) => {
            // A WHATWG URL cannot be constructed without a base. If the
            // provided url is relative and a base is not supplied, the
            // constructor throws an error.
            const _url = new URL(url);
            const params = _url.searchParams;
            if((!"code" in params) || (!"state" in params)) {
                reject(`Invalid redirect params: ${params}`);
            } else {
                const code = params.get("code");
                const state = params.get("state");
                // Here I check if the value set as state during the first
                // authorization call matches the one supplied here in the
                // callback.
                if(auth_data["state"] !== state) {
                    reject(`Saved state (${auth_data["state"]}) doesn't match provided state (${state})`);
                }
                resolve({code, auth_data});
            }
        })
    }
}

async function get_jwks(base_url) {
    // TODO: Probably not all providers use json web key sets.
    // And even if they do they might not expose them at this address.
    // The real address can be discovered via the service metadata endpoint
    // for the token provider.
    return new Promise((resolve, reject) => {
        axios.get(`${base_url}/.well-known/jwks.json`)
            .then(data => resolve(data.data))
            .catch(error => reject(`Error getting jwks: ${error}`))
    })
}

async function verify_token_format(token_response_data) {
    return new Promise((resolve, reject) => {
        if(token_response_data.data === undefined || token_response_data.data === null) {
            reject(`Malformed token response. Missing data. Response: ${token_response_data}`);
        }
        for(const attribute of ["access_token", "token_type", "refresh_token"]) {
            if(!(attribute in token_response_data.data)) {
                reject(`Malformed token response. Missing ${attribute}`);
            }
        }
        resolve(token_response_data.data);
    })
}

async function verify_access_token({header, access_token, refresh_token, jwks}) {
    return new Promise((resolve, reject) => {
        for(const key of jwks["keys"]) {
            if(key["kid"] === header["kid"]) {
                const cert = cert_to_pem(key["x5c"][0]);
                const decoded_token = jsonwebtoken.verify(access_token, cert,
                    {"algorithm": key["alg"],
                    "audience": process.env.OAUTH2_AUDIENCE});
                if(decoded_token.sub === undefined) {
                    console.error("Malformed token", decoded_token);
                    return reject("Token is missing the 'sub' parameter");
                }
                return resolve({access_token, refresh_token, decoded_token});
            }
        }
        reject("No kid matching access token");
    })
}

// This function is taken from here: https://github.com/sgmeyer/auth0-node-jwks-rs256/blob/6d539a8/src/lib/utils.js
// I found it mentioned in these posts:
// https://auth0.com/blog/navigating-rs256-and-jwks/
// https://gist.github.com/westmark/faee223e05bcbab433bfd4ed8e36fb5f
// https://community.auth0.com/t/what-are-the-tradeoffs-between-verifying-jwt-using-public-key-versus-rsa-mod-and-exponentb64/8614
function cert_to_pem(cert) {
    cert = cert.match(/.{1,64}/g).join('\n');
    return `-----BEGIN CERTIFICATE-----\n${cert}\n-----END CERTIFICATE-----\n`;
}

function associate_token_sub_and_user(jwt, client_id, app_host) {
    return async token_data => {
        const {access_token, decoded_token, refresh_token} = token_data;
        return new Promise((resolve, reject) => {
            associate_app_to_user(decoded_token.sub, jwt, client_id, app_host)
                .then(() => resolve({access_token, refresh_token}))
                .catch(error => {
                    console.error(`Error associating user with oauth2 access token: ${error}`);
                    reject("Error associating user with oauth2 access token");
                });
        });
    }
}

function get_token(jwt) {
    return async code_data => {
        const {code, auth_data} = code_data;
        return new Promise((resolve, reject) => {
            const token_data = {
                "grant_type": "authorization_code",
                "code": code,
                "redirect_uri": `${auth_data["redirect_uri"]}/auth/oauth2/callback`,
                "client_id": auth_data["client_id"],
                "code_verifier": auth_data["verifier"],
            }
            const base_url = process.env.OAUTH2_PROVIDER_BASE_URL;
            return get_jwks(base_url).then(jwks => {
                axios.post(`${base_url}/oauth/token`, token_data, {headers: {"content-type": "application/json"}})
                    .then(verify_token_format)
                    .then(tokens => {
                        const {access_token, refresh_token} = tokens;
                        const { header } = jsonwebtoken.decode(access_token,
                            {"complete": true});
                        return {header, access_token, refresh_token, jwks};
                    })
                    .then(verify_access_token)
                    .then(associate_token_sub_and_user(jwt, auth_data["client_id"], auth_data["app_host"]))
                    // To return the access token to a firefox webextension
                    // it has to be in the query parameters it seems. But
                    // the redirect_uri a firefox extension supplies isn't
                    // an address for anything on the internet so it's
                    // probably safe to do it like this (other oauth2
                    // solutions also choose this strategy).
                    // https://faqs.ably.com/is-it-secure-to-send-the-access_token-as-part-of-the-websocket-url-query-params
                    .then(tokens => {
                        const {access_token, refresh_token} = tokens;
                        resolve(`${auth_data["redirect_uri"]}?access_token=${access_token}&refresh_token=${refresh_token}`)
                    })
                    .catch(error => {
                        let error_data = "Unknown";
                        if(error.response !== undefined) {
                            error_data = JSON.stringify(error.response.data);
                            console.error(`Error when querying ${base_url}`, error.response.data);
                        } else {
                            console.error(`Error when querying ${base_url}: ${error}`);
                        }
                        reject(`Error when querying ${base_url}: ${error_data}`);
                    });
            });
        })
    }
}

export async function GET(request) {
    const jwt = await get_jwt();
    return get_userinfo(jwt)
        .then(get_cookie_for_user)
        .then(get_auth_data)
        .then(validate_auth_data)
        .then(get_code(request.url))
        .then(get_token(jwt))
        .then(redirect_uri => {
            return Response.redirect(redirect_uri, 308);
        })
        .catch(error => {
            console.error(`Error during oauth2 callback: ${error}`);
            return new Response(JSON.stringify({"message": "error during oauth2 callback"}), {"headers": {"content-type": "application/json"}, "status": 401});
        })
}
