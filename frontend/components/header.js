"use client"

import Link from "next/link";
import { useRouter } from "next/navigation";

export default function Header() {
    const router = useRouter();

    function logoutOnSubmit(e) {
        e.preventDefault();
        fetch("/api/logout").then(() => {
            router.refresh();
        });
    }

    return (
        <>
            <nav className="navbar">
                <ul className="navbar-nav mr-auto">
                    <li className="nav-item active"><Link href="/">Home</Link></li>
                    <div onClick={logoutOnSubmit}>
                        <li className="nav-item active"><a href="">Logout</a></li>
                    </div>
                </ul>
            </nav>
        </>
    )
}
