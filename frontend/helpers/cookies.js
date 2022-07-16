import nookies from "nookies";

export function set_cookie(request, name, data, max_age = undefined) {
    let options = {
        // If path isn't set here then its value depends on which handler sets it which might make it inaccessible to the other pages
        path: "/",
        httpOnly: true,
        secure: true,
        sameSite: "Strict",
    };
    if(max_age !== undefined) {
        options["maxAge"] = max_age;
    }
    nookies.set(request, name, data, options);
}

export function get_cookie(request) {
    return nookies.get(request);
}

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
