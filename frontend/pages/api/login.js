import axios from "axios";

import { set_cookie } from "../../helpers/cookies.js";

export default async function handler(req, res) {
    return new Promise((resolve, reject) => {
         axios.post(`${process.env.BACKEND_URL}/api/login`,
            {username: req.body.username, password: req.body.password})
            .then(data => {
                set_cookie({res}, "jwt", data.data.jwt)
                res.status(200).json(data.data);
                resolve();
            })
            .catch(error => {
                res.status(500).json({"message": error});
                resolve();
            });
    });
}
