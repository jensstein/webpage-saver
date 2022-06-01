import axios from "axios";

export default async function handler(req, res) {
    const {id} = req.query;
    return new Promise((resolve, reject) => {
         axios.delete(`${process.env.BACKEND_URL}/api/webpage/${id}`, {headers: req.headers})
            .then(_ => {
                res.status(204).send();
                resolve();
            })
            .catch(error => {
                console.log(`Error when deleting webpage ${id}: ${error}`);
                res.status(500).json({"message": `Error when deleting webpage ${id}`});
                resolve();
            });
    });
}
