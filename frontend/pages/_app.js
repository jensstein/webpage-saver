import "bootstrap/dist/css/bootstrap.css";
import '../styles/globals.css'

import { useRouter } from "next/router";
import { createContext, useState, useEffect } from "react";

import { verify_jwt, logout } from "../requests/auth.js";

export const Context = createContext({
    "authorized": false,
});

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
        const path = router.asPath.split("?")[0];
        const remove_jwt = () => {
            logout().then(() => {
                setAuthorized(false);
                sendToLogin(path);
            })
            .catch(_ => console.log("Logout failed"));
        };
        const {jwt, ignore_authentication} = pageProps;
        if(authorized) {
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
        } else if(!ignore_authentication) {
            sendToLogin(path);
        }
    }, [authorized, pageProps]);

    function sendToLogin(path) {
        if(path != "/login") {
            router.push({pathname: "/login", query: {returnUrl: router.asPath.split("?")[0]}});
        }
    }

    return (
        <Context.Provider value={{authorized}}>
            <Component {...pageProps} />
        </Context.Provider>
    )
}

export default MyApp
