import { useRouter } from "next/router";
import { useContext, useEffect, useState } from "react";

import { Context } from "../../_app.js";

import { with_jwt_server_side } from "../../../helpers/auth.js";

export default function Oauth2Callback(props) {
    const [first_run, set_first_run] = useState(true);

    const { authorized } = useContext(Context);
    const router = useRouter();
    useEffect(() => {
        // This is a hack to get access to the correct cookies. When you
        // get to a page from an off-site redirect I don't seem to have
        // access to the jwt cookie. Reloading the page gets that
        // cookie loaded also.
        if(first_run) {
            set_first_run(false);
            router.push(router.asPath);
        } else if(authorized && router.isReady) {
            const params = new URLSearchParams(router.query);
            const url = new URL(`/api/auth/oauth2/callback?${params.toString()}`, window.location.origin);
            router.push(url);
        }
    }, [first_run, authorized]);
    return (
        <p>Handling oauth code</p>
    )
}

export async function getServerSideProps(context) {
    return with_jwt_server_side()(context).then(props => {
        // This tells the base component to ignore the authentication check
        // here and avoid redirecting to the login page. The reason is that
        // on a first load of this page from a source outside of the application
        // you might not have access to the access token cookie even though
        // the user may be logged in.
        props["props"]["ignore_authentication"] = true;
        return props;
    });
}
