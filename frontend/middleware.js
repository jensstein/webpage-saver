import { NextResponse } from "next/server";

import { get_jwt } from "./helpers/cookies.js";
import { verify_jwt } from "./requests/verify-jwt.js";

// https://dev.to/leapcell/implementing-jwt-middleware-in-nextjs-a-complete-guide-to-auth-1b2d
export async function middleware(request) {
    const token = await get_jwt();
    if (!request.nextUrl.pathname.startsWith("/login")) {
        if (!token) {
            return NextResponse.redirect(new URL(`/login?returnUrl=${request.url}`, request.url));
        }

        if (!await verify_jwt(token)) {
            const response = NextResponse.redirect(new URL(`/login?returnUrl=${request.url}`, request.url));
            response.cookies.delete("jwt");
            return response;
        }
    }
    return NextResponse.next();
}

export const config = {
    matcher: ["/((?!api|_next/static|_next/image|favicon.ico).*)"]
}
