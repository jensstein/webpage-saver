"use server"

import { cookies } from "next/headers";

export async function GET(request) {
    cookies().delete("jwt");
    return new Response("");
}
