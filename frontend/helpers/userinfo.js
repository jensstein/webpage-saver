import axios from "axios";

export async function get_userinfo(jwt) {
    return new Promise((resolve, reject) => {
        axios.get(`${process.env.BACKEND_URL}/api/userinfo`,
            {headers: {"Authorization": `bearer ${jwt}`}})
        .then(userinfo => resolve(userinfo.data))
        .catch(error => reject(error));
    })
}
