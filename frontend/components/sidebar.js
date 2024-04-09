"use client"

import { Icon } from "@iconify-icon/react";
import Link from "next/link";
import { useRouter, usePathname, useParams } from "next/navigation";

import { delete_page } from "../requests/delete-page.js";
import { revalidateCallback } from "../helpers/revalidate.js";

export default function Sidebar({jwt, children}) {
    const router = useRouter();
    const pathname = usePathname();
    const {id} = useParams();

    function deleteOnClick(e) {
        e.preventDefault();
        if(id !== undefined && id !== null) {
            delete_page(id, jwt).then(result => {
                if(result) {
                    revalidateCallback();
                    router.push("/");
                } else {
                    alert("Error deleting webpage");
                }
            }).catch(_ => {
                alert("Error deleting webpage");
            });
        } else {
            console.error(`Id param is ${id}`);
        }
    }

    function logoutOnSubmit(e) {
        e.preventDefault();
        fetch("/api/logout").then(() => {
            router.refresh();
        });
    }

    // https://icon-sets.iconify.design/cil/?keyword=core&category=General
    return (
        <>
            <nav className="fixed left-0 top-0 bottom-0 z-50 w-14 bg-sidebar flex flex-col h-screen items-center py-6 rounded-tr-4xl rounded-br-4xl gap-y-10">
                <Link href="/">
                    <Icon icon="cil:home" className="text-2xl" />
                </Link>
                <div onClick={logoutOnSubmit}>
                    <Link href="" target="_blank"><Icon icon="cil:account-logout" className="text-2xl" /></Link>
                </div>
                {pathname.startsWith("/show/") ? (
                    <div onClick={deleteOnClick}>
                        <Link href="" target="_blank">
                            <Icon icon="cil:trash" className="text-2xl" />
                        </Link>
                    </div>
                ) : null}
            </nav>
        </>
    )
}
