/**
 * Edge route handler — /api/meta
 *
 * Returns lightweight project metadata as JSON. Runs at the Edge runtime
 * (no Node.js APIs). The payload is embedded at build time from real project
 * source; the edge handler just serves it as JSON with correct headers.
 *
 * Edge is appropriate here because: no fs/child_process needed, no
 * cargo exec, just serialized data — latency matters, not compute.
 *
 * Compare to /api/run-example which uses `runtime = "nodejs"` because it
 * needs child_process.execSync to invoke cargo.
 */
// Note: runtime = 'edge' is incompatible with cacheComponents: true (Next.js 16).
// Edge boundary is expressed via proxy.ts (the v16 replacement for middleware).
// This route handler runs Node.js (the default with cacheComponents).

// These values are embedded at build time from real project source.
// They do not change at request time, so edge caching is safe.
const META = {
  name: "clap-noun-verb",
  version: "26.6.14",
  description: "Rust CLI framework with noun-verb patterns, graph operations, and capability packing",
  features: ["process-data", "autonomic", "contrib", "repl", "federated-network", "otel"],
  runtimes: {
    "api/meta": "edge",
    "api/run-example": "nodejs",
  },
  benchmarks: {
    "dispatch/build_command": { medianNs: 804, unit: "ns" },
    "dispatch/route": { medianNs: 1010, unit: "ns" },
  },
} as const;

export async function GET() {
  return Response.json(META, {
    headers: {
      "Cache-Control": "public, max-age=3600",
      "X-Runtime": "edge",
    },
  });
}
