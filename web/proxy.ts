/**
 * proxy.ts — Edge boundary for clap-noun-verb dashboard
 *
 * In Next.js 16, `proxy.ts` replaces `middleware.ts` and runs at the Edge
 * by default (before any route renders). This is the correct location for
 * edge-side logic when cacheComponents: true is enabled — the `runtime`
 * route segment config is incompatible with Cache Components.
 *
 * This proxy:
 * 1. Adds a response header `X-Runtime: edge` so the client can observe
 *    which layer handled the request.
 * 2. Adds `X-CNV-Version` — the real project version embedded at build time.
 * 3. Passes all requests through unchanged (no rewrite/redirect).
 *
 * The edge/node boundary is explicit:
 * - proxy.ts runs at the Edge (V8 isolate, Web APIs only)
 * - app/api/run-example/route.ts runs Node.js (needs child_process)
 * - app/api/meta/route.ts runs Node.js (cacheComponents constraint)
 */
import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

export function proxy(_request: NextRequest) {
  const response = NextResponse.next();
  response.headers.set("X-Runtime", "edge");
  response.headers.set("X-CNV-Version", "26.6.14");
  return response;
}

export const config = {
  // Apply to all routes except Next.js internals and static assets.
  matcher: ["/((?!_next/static|_next/image|favicon.ico).*)"],
};
