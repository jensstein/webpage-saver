"use server"

import { cookies } from "next/headers";

export async function GET() {
    cookies().delete("jwt");
    return new Response("");
}
