import styles from '../styles/Login.module.css'

import Header from "../components/header.js";
import { login } from "../requests/auth.js";

import { useRouter } from "next/router";

export default function Login() {
    const router = useRouter();

    function onSubmit(e) {
        e.preventDefault();
        const username = e.target.elements.username.value;
        const password = e.target.elements.password.value;
        login(username, password).then(data => {
            const returnUrl = decodeURIComponent(router.query.returnUrl) || '/';
            router.push(returnUrl);
        }).catch(error => {
            console.log("Error on login: ", error);
        });
    }
    return (
        <>
            <Header/>
            <div className="container py-5 h-100">
                <div className="row d-flex justify-content-center align-items-center h-100">
                    <h2 className="mb-2">Login</h2>
                    <form onSubmit={onSubmit} className={styles.login_box}>
                        <div className="mb-3">
                            <label className="form-label" htmlFor="username-input">username</label>
                            <input id="username-input" className="form-control" name="username" type="text"/>
                        </div>
                        <div className="mb-3">
                            <label className="form-label" htmlFor="password-input">password</label>
                            <input id="password-input" className="form-control" name="password" type="password"/>
                        </div>
                        <button className="btn btn-primary">Login</button>
                    </form>
                </div>
            </div>
        </>
    )
}
