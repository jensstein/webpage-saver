"use client"

import Link from "next/link";

import { useRouter } from "next/navigation";

// Image from next/image cannot be used here since I don't know the domains of all the source urls in advance.
export default function WebpageLink({href, title, image_url}) {
    const router = useRouter();
    if(image_url === null || image_url === undefined) {
        // <div>Icons made by <a href="https://www.freepik.com" title="Freepik">Freepik</a> from <a href="https://www.flaticon.com/" title="Flaticon">www.flaticon.com</a></div>
        image_url = "placeholder.png";
    }

    function onClick() {
        router.push(href);
    }

    return (
        <>
            <div onClick={onClick}>
                <div className="border-solid rounded border-2 flex flex-col text-center justify-center items-center h-60">
                    <img className="h-20 w-auto" src={image_url}/>
                    <Link href={href}><p className="font-light text-lg">{title}</p></Link>
                </div>
            </div>
        </>
    )
}
