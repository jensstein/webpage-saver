"use server"

import axios from "axios";

import { get_headers_from_request } from "../../../helpers/utils.js";

export async function GET(request) {
    const headers = get_headers_from_request(request);
    return axios.get(`${process.env.BACKEND_URL}/api/list-stored-webpages`, {headers})
        .then(data => {
            return new Response(JSON.stringify(data.data), {"headers": {"content-type": "application/json"}});
        })
        .catch(error => {
            console.debug(`Cannot list webpages: ${error}`);
            return new Response(JSON.stringify({"message": "Cannot list webpages"}), {"headers": {"content-type": "application/json"}, "status": 500});
        });
}
