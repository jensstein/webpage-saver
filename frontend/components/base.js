"use client"

import Sidebar from "../components/sidebar.js";

export default function Base({jwt, children}) {
    return (
        <>
            <div className="">
                <Sidebar jwt={jwt} />
                <div className="relative left-14">
                    {children}
                </div>
            </div>
        </>
    )
}
