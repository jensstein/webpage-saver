"use client"

// This is a client component because it needs to set a cookie. This cannot be
// done in a server component so here I handle it by calling a server action
// from this client component.

import { useEffect } from "react";
import { useSearchParams, useRouter } from 'next/navigation'

import { get_authorization_url } from "../../../../helpers/openid/authorize.js";

export default () => {
    const router = useRouter();
    const searchParams = useSearchParams()
    const redirect_uri = searchParams.get("redirect_uri")
    const app_host = searchParams.get("app_host")
    useEffect(() => {
        get_authorization_url(redirect_uri, app_host).then(url => {
            console.debug("Redirecting to OIDC provider", url)
            router.push(url);
        })
    });
    return (
        <>
            <p>Redirecting to OpenID authorization</p>
        </>
    )
}
