import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // House illustrations are served at 90; 75 stays the default for everything else.
  images: { qualities: [75, 90] },
  turbopack: {
    rules: {
      "*.css": {
        loaders: ["@tailwindcss/turbopack"],
        as: "*.css",
      },
    },
  },
};

export default nextConfig;
