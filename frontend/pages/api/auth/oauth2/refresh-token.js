import axios from "axios";

function validate_token_response(response) {
    if(response["data"] === undefined || response["data"] === null) {
        console.error("Malformed refresh token response", response);
        reject("Malformed refresh token response");
    }
    return new Promise((resolve, reject) => {
        for(const attribute of ["access_token", "refresh_token"]) {
            if(!(attribute in response["data"])) {
                reject(`Missing ${attribute} from token response`);
            }
        }
        resolve({"access_token": response["data"]["access_token"],
            "refresh_token": response["data"]["refresh_token"]});
    });
}

export default async function handler(req, res) {
    const base_url = process.env.OAUTH2_PROVIDER_BASE_URL;
    const client_id = process.env.OAUTH2_CLIENT_ID;
    if(req.body.refresh_token === undefined || req.body.refresh_token === null) {
        return res.status(400).json({"message": "Request body missing refresh_token value"});
    }
    const refresh_token = req.body.refresh_token;
    const refresh_token_data = {
        "grant_type": "refresh_token",
        "client_id": client_id,
        "refresh_token": refresh_token,
    }
    axios.post(`${base_url}/oauth/token`, refresh_token_data,
            {headers: {"content-type": "application/json"}})
        .then(validate_token_response)
        .then(response => {
            return res.status(200).json(response);
        })
        .catch(error => {
            let e = error;
            if(error["response"] !== undefined && error["response"] !== null) {
                e = error["response"];
            }
            if(e["data"] !== undefined && e["data"] !== null) {
                e = e["data"];
            }
            console.error("Error refreshing access token", e);
            return res.status(401).send();
        });
}
