import "bootstrap/dist/css/bootstrap.css";
import '../styles/globals.css'

import Base from "../components/base.js";

import { get_jwt } from "../helpers/cookies.js";

export default async function RootLayout(pageProps) {
    const jwt = await get_jwt();

    return (
        <html>
            <body><Base jwt={jwt}>{pageProps.children}</Base></body>
        </html>
    )
}
