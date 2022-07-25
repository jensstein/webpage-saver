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
