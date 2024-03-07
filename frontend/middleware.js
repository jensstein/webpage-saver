import { NextResponse } from "next/server";

import { get_jwt } from "./helpers/cookies.js";

import { verify_jwt } from "./requests/verify-jwt.js";

export async function middleware(request) {
    const jwt = await get_jwt();
    const url = request.nextUrl.clone();
    // NextJS sets the host of the request to be the hostname the next server
    // was started with, not the hostname that the user entered into their browser.
    // https://github.com/vercel/next.js/issues/37536
    const host = request.headers.get("host");
    if(host !== undefined && host !== null) {
        url.host = host;
    }
    const searchParams = new URL(request.url).searchParams;
    const returnUrl = searchParams.get("returnUrl") || encodeURIComponent("/");
    if(jwt === undefined || jwt === null) {
        if(request.nextUrl.pathname !== "/login") {
            return NextResponse.redirect(new URL(`/login?returnUrl=${encodeURIComponent(url)}`, url.origin));
        }
    } else {
        const verified = await verify_jwt(jwt);
        if(verified && request.nextUrl.pathname === "/login") {
            return NextResponse.redirect(returnUrl);
        } else if(!verified) {
            return NextResponse.redirect(new URL(`/login?returnUrl=${encodeURIComponent(url)}`, url.origin));
        }
    }
}

export const config = {
    matcher: ["/", "/show/:page*", "/auth/:page*"],
}
