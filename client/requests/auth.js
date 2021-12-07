import axios from "axios";
import jwt from "jsonwebtoken";

export async function login(username, password) {
    return new Promise((resolve, reject) => {
        axios.post("/api/login", {username, password})
            .then(data => {
                return resolve(data);
            }).catch(error => {
                return reject(`Login failed: ${error}`);
            });
    });
}

export async function verify_jwt(token) {
    const decoded = jwt.decode(token);
    return axios.post("/api/verify-jwt", {username: decoded.sub, "jwt": token})
        .then(_ => {
            return true;
        }).catch(_ => {
            return false;
        });
}
