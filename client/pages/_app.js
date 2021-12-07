import "bootstrap/dist/css/bootstrap.css";
import '../styles/globals.css'

import { useRouter } from "next/router";
import { useState, useEffect } from "react";

import { verify_jwt } from "../requests/auth.js";

function MyApp({ Component, pageProps }) {
    const router = useRouter();
    const [authorized, setAuthorized] = useState(false);

    useEffect(() => {
        const remove_jwt = () => {
            setAuthorized(false);
            router.reload(window.location.pathname);
        };
        const {jwt} = pageProps;
        if(jwt) {
            verify_jwt(jwt).then(is_verified => {
                if(!is_verified) {
                    remove_jwt();
                }
            }).catch(_ => {
                remove_jwt();
            })
        }
        authCheck(router.asPath);
        router.events.on('routeChangeComplete', authCheck)
        return () => {
            router.events.off('routeChangeComplete', authCheck);
        }
    }, [authorized, pageProps]);

    function authCheck(url) {
        const path = url.split("?")[0];
        const {jwt} = pageProps;
        if(authorized) {
            return;
        }
        if((jwt === null || jwt === undefined) && path != "/login") {
            // router.asPath doesn't work down here, for dynamic routes it shows "/show/[id]" instead of "/show/1".
            setAuthorized(false);
            router.push({pathname: "/login", query: {returnUrl: path}});
        } else {
            setAuthorized(true);
        }
    }

    return <Component {...pageProps} />
}

export default MyApp
