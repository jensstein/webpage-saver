import "../styles/globals.css"
import localFont from "next/font/local";

import Base from "../components/base.js";

import { get_jwt } from "../helpers/cookies.js";

// https://nextjs.org/docs/app/getting-started/fonts#local-fonts
const font = localFont({
    src: [{
            path: "../fonts/pt-sans-regular.ttf",
            weight: "400",
            style: "normal",
        }, {
            path: "../fonts/pt-sans-italic.ttf",
            weight: "400",
            style: "italic",
        }, {
            path: "../fonts/pt-sans-bold.ttf",
            weight: "700",
            style: "normal",
        }, {
            path: "../fonts/pt-sans-italic-bold.ttf",
            weight: "700",
            style: "italic",
        }],
});

export default async function RootLayout(pageProps) {
    const jwt = await get_jwt();

    return (
        <html className={font.className}>
            <body><Base jwt={jwt}>{pageProps.children}</Base></body>
        </html>
    )
}
