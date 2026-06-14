import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Enable Cache Components — activates PPR as default rendering model.
  // Static shell (use cache sections) prerendered; dynamic content streams via Suspense.
  cacheComponents: true,
};

export default nextConfig;
