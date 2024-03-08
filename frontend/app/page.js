import { cookies } from "next/headers";
import Link from "next/link";

import styles from '../styles/Home.module.css'

import Header from "../components/header.js";
import LinksContainer from "../components/links-container.js";
import LoadingError from "../components/loading-error.js";
import Loading from "../components/loading.js";
import WebpageLink from "../components/webpage-link.js";
import WebpageFetch from "../components/webpage-fetch.js";

import { revalidateCallback } from "../helpers/revalidate.js";
import {get_jwt} from "../helpers/cookies.js";

export const metadata = {
    "title": "Webpage saver",
    "description": "Show stored webpages",
}

export default async function Home() {
    const jwt = await get_jwt();

    try {
        const result = await fetch(`${process.env.BACKEND_URL}/api/list-stored-webpages`,
            {"headers": {"authorization": `Bearer ${jwt}`}, "next":
                {"tags": ["stored-webpages"], "revalidate": 3600}
            });
        const data = await result.json();

        let webpage_ids = data.webpage_infos.map(({id, title, image_url}, i) => {
            return (
                <WebpageLink key={`webpage-${i}`} href={`/show/${id}`} title={title} image_url={image_url}/>
            )
        });
        return (
            <div className={styles.container}>
                <main>
                    <Header/>
                    <h1 className={styles.title}>Article saver</h1>
                </main>
                <WebpageFetch jwt={jwt} revalidateCallback={revalidateCallback}/>
                <LinksContainer>
                    {webpage_ids}
                </LinksContainer>
            </div>
        )
    } catch(error) {
        console.error(`Error fetching data: ${error}`)
        return (
            <LoadingError/>
        )
    }
}
