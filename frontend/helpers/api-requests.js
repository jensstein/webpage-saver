import axios from "axios";

import { check_value } from "../helpers/utils.js";

export class RequestSender {
    constructor(request) {
        this.auth_type = request.query["auth-type"];
    }

    get(url, headers) {
        return this.__request(url, "get", undefined, headers);
    }

    post(url, data, headers) {
        return this.__request(url, "post", data, headers);
    }

    __request(url, method, data, headers) {
        const _url = new URL(url, process.env.SERVER_BASE_URL);
        // If the auth-type query parameter is set here it should be
        // propagated to the backend call as well.
        if(!_url.searchParams.has("auth-type") && check_value(this.auth_type)) {
            _url.searchParams.set("auth-type", this.auth_type);
        }
        return axios({
            method,
            url: _url.toString(),
            data,
            headers,
        })
    }
}
