import axios from "axios";

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

export async function logout() {
    return new Promise((resolve, reject) => {
        axios.get("/api/logout")
            .then(data => {
                return resolve();
            }).catch(error => {
                return reject(`Logout failed: ${error}`);
            });
    });
}
