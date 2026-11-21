import { defineConfig } from "astro/config";

const isGitHubPages = process.env.GITHUB_PAGES === "true";
const cloudflarePreviewUrl =
  process.env.CF_PAGES === "1" && process.env.CF_PAGES_BRANCH !== "main"
    ? process.env.CF_PAGES_URL
    : undefined;
const site =
  cloudflarePreviewUrl ??
  process.env.SITE_URL ??
  (isGitHubPages ? "https://oguzbay-del.github.io" : "https://gristora.com");
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
