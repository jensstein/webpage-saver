import Head from 'next/head'
import styles from '../styles/Home.module.css'

import Header from "../components/header.js";
import LinksContainer from "../components/links-container.js";
import LoadingError from "../components/loading-error.js";
import Loading from "../components/loading.js";
import WebpageLink from "../components/webpage-link.js";
import WebpageFetch from "../components/webpage-fetch.js";

import {get_jwt} from "../helpers/cookies.js";

import Link from "next/link";
import useSWR, {useSWRConfig} from "swr";

export default function Home({jwt}) {
    const { mutate } = useSWRConfig();

    const fetcher = url => {
        if(jwt === null || jwt === undefined) {
            return {};
        }
        return fetch(url, {headers: {"Authorization": `Bearer ${jwt}`}}).then(res => res.json());
    };
    const {data, error} = useSWR("/api/list-webpages", fetcher, {
        revalidateOnFocus: false,
    });

    // https://swr.bootcss.com/en-US/docs/mutation
    const revalidateCallback = () => mutate("/api/list-webpages");

    if(error) {
        return (
            <LoadingError/>
        )
    } else if(!data || data.webpage_infos === undefined) {
        return (
            <Loading/>
        )
    }

    let webpage_ids = data.webpage_infos.map(({id, title, image_url}, i) => {
        return (
            <WebpageLink key={`webpage-${i}`} href={`/show/${id}`} title={title} image_url={image_url}/>
        )
    });
    return (
        <div className={styles.container}>
            <Head>
              <title>Webpage saver</title>
              <meta name="description" content="Show stored webpages"/>
            </Head>
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
}

export async function getServerSideProps({req, res}) {
    const jwt = get_jwt({req});
    return {props: {
        jwt,
    }};
}
