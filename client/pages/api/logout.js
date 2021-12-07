import nookies from "nookies";

export default async function handler(req, res) {
    nookies.set({res}, "jwt", "removed", {
        path: "/",
        httpOnly: true,
        secure: true,
        sameSite: "Strict",
        // maxAge: 0 makes the cookies get deleted immediately
        maxAge: "0",
    });
    res.status(200).send();
}
