import axios from "axios";

export default async function handler(req, res) {
    return new Promise((resolve, reject) => {
        const {authorization} = req.headers;
        return axios.post(`${process.env.BACKEND_URL}/api/associate-app-to-user`,
                {sub: req.body.sub, client_id: req.body.client_id, app_host: req.body.app_host},
                {headers: {authorization}})
            .then(data => res.status(201).send())
            .catch(error => {
                if(error.response !== undefined) {
                    console.error("Error when associating user with app: ", error.response.status, error.response.data);
                    if(error.response.status === 400) {
                        res.status(400).json(error.response.data);
                    }
                } else {
                    console.error("Error when associating user with app: ${error}");
                    res.status(500).json({message: "Unable to associate user with app"});
                }
                reject();
            });
        resolve();
    });
}
