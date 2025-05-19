import '../styles/globals.css'
import {  PT_Sans  } from "next/font/google";

import Base from "../components/base.js";

import { get_jwt } from "../helpers/cookies.js";

const font = PT_Sans({ subsets: ["latin"], weight: ["400", "700"] });

export default async function RootLayout(pageProps) {
    const jwt = await get_jwt();

    return (
        <html className={font.className}>
            <body><Base jwt={jwt}>{pageProps.children}</Base></body>
        </html>
    )
}
