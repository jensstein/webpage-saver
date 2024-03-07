"use server"

import axios from "axios";

export async function POST(request) {
    const body = await request.json();
    return axios.post(`${process.env.BACKEND_URL}/api/verify-jwt`,
        {username: body.username, jwt: body.jwt})
        .then(data => {
            return new Response(JSON.stringify(data.data), {"headers": {"content-type": "application/json"}});
        })
        .catch(error => {
            console.debug(`Error verifying JWT: ${error}`);
            return new Response(JSON.stringify({"message": "Cannot log in"}), {"headers": {"content-type": "application/json"}, "status": 500});
        });
}
