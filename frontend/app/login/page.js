"use client"

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
        login(username, password).then(() => {
            router.push(decodeURIComponent(searchParams.get("returnUrl") || '/'));
        }).catch(error => {
            console.log("Error on login: ", error);
        });
    }
    // Inspired by https://tailwindcomponents.com/component/login-form-31
    return (
        <div className="grid h-screen place-items-center">
            <div className="bg-white p-8 rounded shadow-md max-w-lg w-full mx-auto">
                <div className="grid grid-cols-1 gap-4">
                    <h2 className="mb-2">Login</h2>
                    <form onSubmit={onSubmit} className="max-w-sm">
                        <div className="mb-3">
                            <label className="block text-sm font-medium text-gray-700" htmlFor="username-input">username</label>
                            <input id="username-input" className="mt-1 p-2 w-full border rounded-md" name="username" type="text"/>
                        </div>
                        <div className="mb-3">
                            <label className="block text-sm font-medium text-gray-700" htmlFor="password-input">password</label>
                            <input id="password-input" className="mt-1 p-2 w-full border rounded-md" name="password" type="password"/>
                        </div>
                        <button className="btn-primary">Login</button>
                    </form>
                </div>
            </div>
        </div>
    )
}
