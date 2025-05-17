"use server"

import { cookies } from "next/headers";

export async function GET() {
    (await cookies()).delete("jwt");
    return new Response("");
}
