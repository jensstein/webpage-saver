import styles from "../styles/Sidebar.module.css";

import { useRouter } from "next/router";

export default function Sidebar({jwt}) {
    const router = useRouter();
    const {id} = router.query;

    function deleteOnClick(e) {
        e.preventDefault();
        fetch(`/api/delete-webpage/${id}`, {headers: {"Authorization": `Bearer ${jwt}`}}).then(_ => {
            router.push("/");
        }).catch(_ => {
            alert("Error when deleting webpage");
        });
    }
    return (
        <>
            <nav id={styles.sidebar}>
                <ul className="nav nav-pills flex-column mb-auto">
                    <li>
                        <a className="nav-link link-dark" href="#" onClick={deleteOnClick}>
                            {/* Using Image from next/image here makes styling
                              * a bit hard since Image includes wrappers around
                              * the img element but only applies the styling to
                              * that inner element.
                              * https://github.com/vercel/next.js/issues/18585 */}
                            <img className={styles.sidebar_icon + " bi me-2"} src="/assets/img/trash.svg" alt="delete webpage" title="delete webpage"/>
                            <span className={styles.sidebar_text}>Delete webpage</span>
                        </a>
                    </li>
                </ul>
            </nav>
        </>
    )
}
