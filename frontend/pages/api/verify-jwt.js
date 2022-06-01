import axios from "axios";

export default async function handler(req, res) {
    return new Promise((resolve, reject) => {
         axios.post(`${process.env.BACKEND_URL}/api/verify-jwt`,
            {username: req.body.username, jwt: req.body.jwt})
            .then(data => {
                res.status(200).json(data.data);
                resolve();
            })
            .catch(error => {
                res.status(500).json({"message": error});
                reject();
            });
    });
}
