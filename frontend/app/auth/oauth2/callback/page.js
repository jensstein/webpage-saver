import { redirect } from 'next/navigation'
import { headers } from "next/headers";

import * as client from "openid-client";
import jsonwebtoken from "jsonwebtoken";

import { get_cookie } from "../../../../helpers/cookies.js";

export default async (request) => {
    const headers_list = await headers();
    const protocol = headers_list.get("X-Forwarded-Proto") || "https";
    const host = `${protocol}://${headers_list.get("host")}`;

    const oidc_session_cookie = await get_cookie("__oidc_session");
    const oidc_session = JSON.parse(oidc_session_cookie.value);

    const searchParams = await request.searchParams;
    const code = searchParams["code"];
    const state = searchParams["state"];
    const redirect_uri = searchParams["redirect_uri"];
    let config = await client.discovery(
        new URL(process.env.OAUTH2_PROVIDER_BASE_URL),
        process.env.OAUTH2_CLIENT_ID,
    )
    let tokens = null;
    let error = false;
    try {
        tokens = await client.authorizationCodeGrant(
            config,
            new URL(`${host}/auth/oauth2/callback?code=${code}`), {
                pkceCodeVerifier: oidc_session["code_verifier"],
                expectedNonce: oidc_session["nonce"],
            },
        );
        const { payload } = jsonwebtoken.decode(tokens.access_token,
            {"complete": true});
        const response = await fetch(`${process.env.BACKEND_URL}/api/associate-app-to-user?auth-type=oauth2`, {
                method: "POST",
                body: JSON.stringify({
                    sub: payload["sub"], client_id: process.env.OAUTH2_CLIENT_ID,
                    app_host: oidc_session["app_host"]}),
                headers: {"authorization": `Bearer ${tokens.access_token}`,
                    "content-type": "application/json"}
            });
        if(!response.ok) {
            error = true;
            const body = await response.text();
            return (
                <>
                    <p>Could not associate OpenID sub to user: {body}</p>
                </>
            )
        }
    } catch(exception) {
        console.error(`Caught error when fetching openid tokens ${exception}`);
    } finally {
        // redirect must not be called in the try block since it throws an error internally: https://nextjs.org/docs/app/building-your-application/routing/redirecting#redirect-function
        if(tokens !== null && !error) {
            redirect(`${redirect_uri}?access_token=${tokens.access_token}&refresh_token=${tokens.refresh_token}`)
        }
    }
}
