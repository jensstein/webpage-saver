"use server"

import { cookies } from "next/headers";

export async function set_cookie(name, data, max_age = undefined) {
    let options = {
        // If path isn't set here then its value depends on which handler sets it which might make it inaccessible to the other pages
        path: "/",
        httpOnly: true,
        secure: process.env.NODE_ENV !== "development",
        sameSite: "Strict",
    };
    if(max_age !== undefined) {
        options["maxAge"] = max_age;
    }
    await cookies().set(name, data, options);
}

export async function get_cookie(request) {
    return cookies().get(request);
}

export async function get_jwt() {
    let jwt = await get_cookie("jwt");
    if(jwt === undefined) {
        return null;
    }
    return jwt.value;
}
