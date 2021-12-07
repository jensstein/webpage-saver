import axios from "axios";

import nookies from "nookies";

export default async function handler(req, res) {
    return new Promise((resolve, reject) => {
         axios.post(`${process.env.BACKEND_URL}/api/login`,
            {username: req.body.username, password: req.body.password})
            .then(data => {
                nookies.set({res}, "jwt", data.data.jwt, {
                    // If path isn't set here then its value is /api which makes it inaccessible to the other pages
                    path: "/",
                    httpOnly: true,
                    secure: true,
                    sameSite: "Strict",
                });
                res.status(200).json(data.data);
                resolve();
            })
            .catch(error => {
                res.status(500).json({"message": error});
                resolve();
            });
    });
}
