import "bootstrap/dist/css/bootstrap.css";
import '../styles/globals.css'

import { useRouter } from "next/router";
import { useState, useEffect } from "react";

import { verify_jwt, logout } from "../requests/auth.js";

function MyApp({ Component, pageProps }) {
    const router = useRouter();
    const [authorized, setAuthorized] = useState(false);

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
                router.push({pathname: "/login", query: {returnUrl: router.asPath.split("?")[0]}});
            })
            .catch(_ => console.log("Logout failed"));
        };
        const {jwt} = pageProps;
        if(jwt) {
            verify_jwt(jwt).then(is_verified => {
                if(!is_verified) {
                    remove_jwt();
                } else {
                    setAuthorized(true);
                }
            }).catch(_ => {
                remove_jwt();
            })
        }
        authCheck(router.asPath);
        router.events.on("routeChangeComplete", authCheck)
        return () => {
            router.events.off("routeChangeComplete", authCheck);
        }
    }, [authorized, pageProps]);

    function authCheck(url) {
        if(authorized) {
            return;
        }
        const path = url.split("?")[0];
        const {jwt} = pageProps;
        if((jwt === null || jwt === undefined) && path != "/login") {
            // router.asPath doesn't work down here, for dynamic routes it shows "/show/[id]" instead of "/show/1".
            router.push({pathname: "/login", query: {returnUrl: path}});
        }
    }

    return <Component {...pageProps} />
}

export default MyApp
