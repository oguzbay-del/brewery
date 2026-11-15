import { defineConfig } from "astro/config";

const isGitHubPages = process.env.GITHUB_PAGES === "true";
const vercelHost = process.env.VERCEL_URL;
const site =
  process.env.SITE_URL ??
  (isGitHubPages
    ? "https://oguzbay-del.github.io"
    : vercelHost
      ? `https://${vercelHost}`
      : "https://gristora.com");
const base = process.env.SITE_BASE ?? (isGitHubPages ? "/brewery" : "/");

export default defineConfig({
  site,
  base,
  trailingSlash: "always",
  output: "static",
  build: {
    format: "directory",
  },
});
