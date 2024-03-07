"use client"

import { useRouter, usePathname } from "next/navigation";
import { useState, useEffect } from "react";

import { logout } from "../requests/auth.js";
import { verify_jwt } from "../requests/verify-jwt.js";

import { get_jwt } from "../helpers/cookies.js";

export default function Base({jwt, children}) {
    const router = useRouter();
    const [authorized, setAuthorized] = useState(false);

    const path = usePathname().split("?")[0];

    // Be careful of infinite loops here. If no dependencies are set,
    // useEffect will retrigger on any state change which then again
    // triggers a re-render. If dependencies are set, useEffect will
    // trigger any time they change so infinite loops are still possible
    // here if you make a mistake.
    // https://dmitripavlutin.com/react-useeffect-infinite-loop/
    useEffect(() => {
        const remove_jwt = () => {
            logout().then(() => {
                setAuthorized(false);
                sendToLogin(path);
            })
            .catch(_ => console.log("Logout failed"));
        };
        if(authorized && (jwt !== undefined && jwt !== null)) {
            return;
        } else if(jwt) {
            verify_jwt(jwt).then(is_verified => {
                if(!is_verified) {
                    remove_jwt();
                } else {
                    setAuthorized(true);
                }
            }).catch(_ => {
                remove_jwt();
            })
        } else {
            sendToLogin(path);
        }
    }, [authorized, jwt, children]);

    function sendToLogin(path) {
        if(path != "/login") {
            const p = `/login?returnUrl=${encodeURIComponent(path)}`;
            router.push(p);
        }
    }

    return (
        <>{children}</>
    )
}
