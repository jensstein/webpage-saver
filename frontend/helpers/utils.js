export function check_value(value) {
    return value !== undefined && value !== null;
}

export function get_info_from_request_error(error) {
    let info = {
        "status": 0,
        "message": "",
        "data": "",
    };
    if(check_value(error.response)) {
        if(check_value(error.response.status)) {
            info["status"] = error.response.status;
        }
        if(check_value(error.response.statusText)) {
            info["message"] = error.response.statusText;
        }
        if(check_value(error.response.data)) {
            info["data"] = error.response.data;
        }
    }
    return info;
}

export function get_headers_from_request(request) {
    const headers = new Headers;
    for (const h of request.headers.entries()) {
        headers.set(h[0], h[1]);
    }
    return headers;
}

class Headers {
    constructor() {
        this.headers = {};
    }

    get(key) {
        return this.headers[key.toLowerCase()];
    }

    set(key, value) {
        this.headers[key.toLowerCase()] = value;
    }
}
