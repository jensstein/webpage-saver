"use client"

import styles from '../../styles/Login.module.css'

import { login } from "../../requests/auth.js";

import { useRouter, useSearchParams } from "next/navigation";

import { useEffect } from "react";

import { verify_jwt } from "../../requests/verify-jwt.js";

import { get_jwt } from "../../helpers/cookies.js";

export default function Login() {
    const router = useRouter();

    const searchParams = useSearchParams();

    useEffect(() => {
        get_jwt().then(jwt => {
            if(jwt !== undefined && jwt !== null) {
                verify_jwt(jwt).then(result => {
                    if(result) {
                        router.push(decodeURIComponent(searchParams.get("returnUrl") || '/'));
                    }
                });
            }
        });
    });

    function onSubmit(e) {
        e.preventDefault();
        const username = e.target.elements.username.value;
        const password = e.target.elements.password.value;
        login(username, password).then(data => {
            router.push(decodeURIComponent(searchParams.get("returnUrl") || '/'));
        }).catch(error => {
            console.log("Error on login: ", error);
        });
    }
    return (
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
    )
}
