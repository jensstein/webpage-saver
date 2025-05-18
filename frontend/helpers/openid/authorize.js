"use server"

import { redirect } from "next/navigation"
import { headers } from "next/headers";

import * as client from "openid-client";
import { set_cookie } from "../cookies.js";
import { cookies } from "next/headers";

export async function get_authorization_url(redirect_uri, app_host) {
    const headers_list = await headers();
    const protocol = headers_list.get("X-Forwarded-Proto") || "https";
    const host = `${protocol}://${headers_list.get("host")}`;

    // https://github.com/panva/openid-client
    let config = await client.discovery(
        new URL(process.env.OAUTH2_PROVIDER_BASE_URL),
        process.env.OAUTH2_CLIENT_ID,
    )
    let code_verifier = client.randomPKCECodeVerifier()
    let code_challenge =
        await client.calculatePKCECodeChallenge(code_verifier)
    const nonce = client.randomNonce();

    set_cookie(await cookies(), "__oidc_session",
        JSON.stringify({code_verifier, nonce, app_host}), 600, "Lax");
    let parameters = {
        redirect_uri: new URL(`${host}/auth/oauth2/callback?redirect_uri=${redirect_uri}`),
        scope: "openid offline_access",
        code_challenge,
        code_challenge_method: "S256",
        nonce,
        audience: process.env.OAUTH2_AUDIENCE,
    }
    let redirectTo = client.buildAuthorizationUrl(config, parameters)
    return redirectTo.href;
}
