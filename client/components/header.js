import Link from "next/link";
import { useRouter } from "next/router";

export default function Header() {
    const router = useRouter();

    function logoutOnSubmit(e) {
        e.preventDefault();
        fetch("/api/logout").then(() => {
            router.reload(window.location.pathname);
        });
    }

    return (
        <>
            <nav className="navbar">
                <ul className="navbar-nav mr-auto">
                    <li className="nav-item active"><Link href="/"><a>Home</a></Link></li>
                    <div onClick={logoutOnSubmit}>
                        <li className="nav-item active"><a href="">Logout</a></li>
                    </div>
                </ul>
            </nav>
        </>
    )
}
