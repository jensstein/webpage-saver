"use client"

import axios from "axios";

export default function WebpageFetch({jwt, revalidateCallback}) {
    function submit(e) {
        e.preventDefault();
        const url = e.target.elements["webpage-url-input"].value;
        axios.post("/api/fetch", {url}, {headers: {"Authorization": `bearer ${jwt}`}})
            .then(data => {
                revalidateCallback();
            })
            .catch(error => {
                console.error(`Unable to fetch ${url}: ${error}`)
                alert(`Error fetching page ${url}`);
            });
    }

    return (
        <form onSubmit={submit}>
            <fieldset className="w-full space-y-1 dark:text-gray-100">
                <label htmlFor="webpage-url-input" className="block text-sm font-medium">Fetch webpage by url: </label>
                <input type="text" name="url" id="webpage-url-input" className="hur pl-5 flex flex-1 border sm:text-sm rounded-r-md focus:ri dark:border-gray-700 dark:text-gray-100 dark:bg-gray-800 focus:ri" />
            </fieldset>
        </form>
    )
}
