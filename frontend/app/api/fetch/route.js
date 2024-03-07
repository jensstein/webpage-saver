"use server"

import axios from "axios";

import { get_headers_from_request, get_info_from_request_error } from "../../../helpers/utils.js";

export async function POST(request) {
    const body = await request.json();
    const searchParams = new URL(request.url).searchParams;
    const headers = get_headers_from_request(request);
    if(headers.get("authorization") === undefined || headers.get("authorization") === null) {
        return new Response(JSON.stringify({message: "Missing authorization header"}), {"headers": {"content-type": "application/json"}, "status": 401});
    }
    let post_data = {url: body.url};
    if(body.html !== undefined && body.html !== null) {
        post_data["html"] = body.html;
    }
    const post_headers = {"Authorization": headers.get("authorization")};
    const auth_type = searchParams.get("auth-type");
    if(auth_type !== undefined && auth_type !== null) {
        post_headers["auth-type"] = auth_type;
    }
    return axios.post(`${process.env.BACKEND_URL}/api/fetch`,
        post_data, {"headers": post_headers})
        .then(data => {
            return new Response("", {"status": 200});
        })
        .catch(error => {
            const error_info = get_info_from_request_error(error);
            console.log(`Error happended when fetching ${body.url}: ${error}`, error_info);
            let status = 500;
            if(error.response !== undefined && error.response.status !== undefined) {
                status = error.response.status
            }
            return new Response("", {"status": status});
        });
}
