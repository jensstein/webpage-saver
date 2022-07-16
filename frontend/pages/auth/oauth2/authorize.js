import { useRouter } from "next/router";
import { useContext, useEffect } from "react";

import { Context } from "../../_app.js";

import { with_jwt_server_side } from "../../../helpers/auth.js";

export default function Oauth2Connect() {
    const { authorized } = useContext(Context);
    const router = useRouter();
    useEffect(() => {
        if(authorized && router.isReady) {
            const { redirect_uri, app_host } = router.query;
            let url = "/api/auth/oauth2/authorize";
            if(redirect_uri !== undefined && redirect_uri !== null) {
                url = `${url}?redirect_uri=${redirect_uri}&app_host=${app_host}`;
            }
            router.push(url);
        }
    })
    return (
        <p>Redirecting to oauth2 login flow</p>
    )
}

export const getServerSideProps = with_jwt_server_side();
