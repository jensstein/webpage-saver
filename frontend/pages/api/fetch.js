import { RequestSender } from "../../helpers/api-requests.js";
import { get_info_from_request_error } from "../../helpers/utils.js";

export default async function handler(req, res) {
    const sender = new RequestSender(req);
    return new Promise((resolve, reject) => {
        if(req.headers.authorization === undefined || req.headers.authorization === null) {
            res.status(401).json({message: "Missing authorization header"});
            return resolve();
        }
        let body = {url: req.body.url};
        if(req.body.html !== undefined && req.body.html !== null) {
            body["html"] = req.body.html;
        }
        sender.post(`${process.env.BACKEND_URL}/api/fetch`,
            body, {"Authorization": req.headers.authorization})
            .then(data => {
                res.status(data.status).send();
                resolve();
            })
            .catch(error => {
                const error_info = get_info_from_request_error(error);
                console.log(`Error happended when fetching ${req.body.url}: ${error}`, error_info);
                let status = 500;
                if(error.response !== undefined && error.response.status !== undefined) {
                    status = error.response.status
                }
                res.status(status).send();
                resolve();
            });
    });
}

// https://nextjs.org/docs/api-routes/request-helpers#custom-config
export const config = {
    api: {
        bodyParser: {
            // The default limit is 1mb. This is too little for some webpages.
            // In order to avoid implementing batch uploading, I raise the limit to 10mb.
            // next.js gives the error `413 Body exceeded 1mb limit` when too much data is posted.
            sizeLimit: "10mb"
        }
    }
}
