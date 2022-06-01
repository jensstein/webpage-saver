import nookies from "nookies";

export function get_jwt(request) {
    let {jwt} = nookies.get(request, "jwt");
    if(jwt === undefined) {
        return null;
    }
    return jwt;
}

export function remove_cookie(context, name) {
    nookies.destroy(context, name, {path: "/"});
}
