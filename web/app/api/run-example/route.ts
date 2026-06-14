import { NextRequest, NextResponse } from "next/server";
import { runExample } from "@/lib/project";

// Node.js runtime (default). runtime segment config removed — incompatible
// with cacheComponents: true. Node.js is the default and the correct runtime
// here since runExample() uses child_process.execSync.

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
