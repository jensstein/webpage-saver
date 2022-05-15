import styles from "../../styles/ShowStoredWebpage.module.css";

import Header from "../../components/header.js";
import LoadingError from "../../components/loading-error.js";
import Loading from "../../components/loading.js";

import {get_jwt} from "../../helpers/cookies.js";

import { useRouter } from "next/router";
import useSWR from "swr";

export default function ShowStoredWebpage({jwt}) {
    const router = useRouter();
    const {id} = router.query;

    const fetcher = url => {
        return fetch(url, {headers: {"Authorization": `Bearer ${jwt}`}}).then(res => res.json());
    };
    const {data, error} = useSWR(`/api/show-webpage/${id}`, fetcher, {
        revalidateOnFocus: false,
    });

    if(error) {
        return <LoadingError/>;
    } else if(!data) {
        return <Loading/>;
    }

    return (
        <>
            <Header/>
            <div className={styles.webpage_container}>
                <h1>{data.title}</h1>
                <div id={styles.content} dangerouslySetInnerHTML={{__html: data.content}}/>
            </div>
        </>
    )
}

export async function getServerSideProps({req, res}) {
    const jwt = get_jwt({req});
    return {props: {
        jwt,
    }};
    return {props: {
        jwt,
    }};
}
