"use server"

import axios from "axios";

import { set_cookie } from "../../../helpers/cookies.js";

export async function POST(request) {
    const body = await request.json();
    return axios.post(`${process.env.BACKEND_URL}/api/login`,
        {username: body.username, password: body.password})
        .then(data => {
            set_cookie("jwt", data.data.jwt).catch(
                error => console.error(`Error setting cookie: ${error}`)
            );
            return new Response(JSON.stringify(data.data), {"headers": {"content-type": "application/json"}});
        })
        .catch(error => {
            console.debug(`Error logging in: ${error}`);
            return new Response(JSON.stringify({"message": "Cannot log in"}), {"headers": {"content-type": "application/json"}, "status": 500});
        });
}
