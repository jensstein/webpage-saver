"use server"

export async function delete_page(id, jwt) {
    return await fetch(`${process.env.BACKEND_URL}/api/webpage/${id}`,
            {"method": "DELETE",
            "headers": {"Authorization": `Bearer ${jwt}`}})
        .then(result => {
            return result.ok;
        }).catch(error => {
            console.error(`Error deleting ${id}: ${error}`);
            return false;
        });
}
