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
        <>
            <div>
                <form onSubmit={submit}>
                    <div>
                        <label className="form-label" htmlFor="webpage-url-input">Fetch webpage by url: </label>
                        <input id="webpage-url-input"/>
                    </div>
                </form>
            </div>
        </>
    )
}

