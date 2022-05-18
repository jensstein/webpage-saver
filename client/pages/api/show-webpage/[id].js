import axios from "axios";

export default async function handler(req, res) {
    const {id} = req.query;
    return new Promise((resolve, reject) => {
         axios.get(`${process.env.BACKEND_URL}/api/webpage/${id}`, {headers: req.headers})
            .then(data => {
                res.status(200).json(data.data);
                resolve();
            })
            .catch(error => {
                res.status(500).json({"message": error});
                resolve();
            });
    });
}

