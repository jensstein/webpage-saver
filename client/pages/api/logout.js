import { remove_cookie } from "../../helpers/cookies.js";

export default async function handler(req, res) {
    return new Promise((resolve, reject) => {
        remove_cookie({res}, "jwt");
        res.status(200).send();
        resolve();
    });
}
