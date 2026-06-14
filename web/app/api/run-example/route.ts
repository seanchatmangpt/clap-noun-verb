import { NextRequest, NextResponse } from "next/server";
import { runExample } from "@/lib/project";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function GET(req: NextRequest) {
  const name = req.nextUrl.searchParams.get("name");
  if (!name || !/^[a-z0-9_-]+$/.test(name)) {
    return NextResponse.json({ error: "invalid example name" }, { status: 400 });
  }
  try {
    const result = runExample(name);
    return NextResponse.json(result);
  } catch (e: unknown) {
    const err = e as { message?: string };
    return NextResponse.json({ error: err.message ?? String(e) }, { status: 500 });
  }
}
