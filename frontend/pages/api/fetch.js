import { RequestSender } from "../../helpers/api-requests.js";

export default async function handler(req, res) {
    const sender = new RequestSender(req);
    return new Promise((resolve, reject) => {
        if(req.headers.authorization === undefined || req.headers.authorization === null) {
            res.status(401).json({message: "Missing authorization header"});
            return resolve();
        }
        sender.post(`${process.env.BACKEND_URL}/api/fetch`,
            {url: req.body.url}, {"Authorization": req.headers.authorization})
            .then(data => {
                res.status(data.status).send();
                resolve();
            })
            .catch(error => {
                console.log(`Error happended when fetching ${req.body.url}: ${error}`);
                let status = 500;
                if(error.response !== undefined && error.response.status !== undefined) {
                    status = error.response.status
                }
                res.status(status).send();
                resolve();
            });
    });
}
