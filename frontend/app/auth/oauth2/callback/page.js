"use client"

import { useRouter, usePathname, useSearchParams } from "next/navigation";
import { useEffect, useState } from "react";

import { get_jwt } from "../../../../helpers/cookies.js";

export default function Oauth2Callback(props) {
    const [first_run, set_first_run] = useState(true);
    const path = usePathname().split("?")[0];
    const searchParams = useSearchParams();

    const router = useRouter();
    useEffect(() => {
        get_jwt().then(result => {
            // This is a hack to get access to the correct cookies. When you
            // get to a page from an off-site redirect I don't seem to have
            // access to the jwt cookie. Reloading the page gets that
            // cookie loaded also.
            if(first_run) {
                set_first_run(false);
                router.push(path);
            } else {
                router.push(`/api/auth/oauth2/callback?${searchParams.toString()}`);
            }
        });
    }, [first_run]);
    return (
        <p>Handling oauth code</p>
    )
}
