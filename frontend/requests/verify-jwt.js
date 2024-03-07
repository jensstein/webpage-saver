"use server"

import jwt from "jsonwebtoken";

export async function verify_jwt(token) {
    const decoded = jwt.decode(token);
    return await fetch(`${process.env.BACKEND_URL}/api/verify-jwt`,
            {"method": "POST", "body": JSON.stringify(
                {"username": decoded.sub, "jwt": token}),
            "headers": {"content-type": "application/json"}})
        .then(result => {
            return result.ok;
        }).catch(error => {
            return false;
        });
}
