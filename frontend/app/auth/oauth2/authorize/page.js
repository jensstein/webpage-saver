"use client"

import { useRouter, useSearchParams } from "next/navigation";
import { useEffect } from "react";

import { get_jwt } from "../../../../helpers/cookies.js";

export default function Oauth2Connect() {
    const searchParams = useSearchParams();
    const router = useRouter();
    useEffect(() => {
        get_jwt().then(result => {
            const redirect_uri = searchParams.get("redirect_uri")
            const app_host = searchParams.get("app_host");
            let url = "/api/auth/oauth2/authorize";
            if(redirect_uri !== undefined && redirect_uri !== null) {
                url = `${url}?redirect_uri=${redirect_uri}&app_host=${app_host}`;
            }
            router.push(url);
        });
    });
    return (
        <p>Redirecting to oauth2 login flow</p>
    )
}
