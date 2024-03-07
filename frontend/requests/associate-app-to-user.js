"use server"

import axios from "axios";

export async function associate_app_to_user(sub, jwt, client_id, app_host) {
    return axios.post(`${process.env.BACKEND_URL}/api/associate-app-to-user`,
        {sub: sub, client_id: client_id, app_host: app_host},
        {headers: {"authorization": `Bearer ${jwt}`}});
}
