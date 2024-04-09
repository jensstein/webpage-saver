"use server"

import styles from "../../../styles/ShowStoredWebpage.module.css";

import LoadingError from "../../../components/loading-error.js";
import Loading from "../../../components/loading.js";
import Sidebar from "../../../components/sidebar.js";

import { get_jwt } from "../../../helpers/cookies.js";

export default async function ShowStoredWebpage({params}) {
    const {id} = params;
    const jwt = await get_jwt();
    const f = await fetch(`${process.env.BACKEND_URL}/api/webpage/${id}`, {headers: {"authorization": `Bearer ${jwt}`}});
    if(f.status >= 400) {
        return <LoadingError/>;
    }
    try {
        const data = await f.json();

        return (
            <div id={styles.wrapper}>
                <div className={styles.webpage_container}>
                    <h1 className="text-3xl font-semibold">{data.title}</h1>
                    <div id={styles.content} dangerouslySetInnerHTML={{__html: data.content}}/>
                </div>
            </div>
        )
    } catch(error) {
        console.error(`Error showing page: ${error}`);
        return <LoadingError/>;
    }
}
