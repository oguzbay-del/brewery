import { defineConfig } from "astro/config";

const isGitHubPages = process.env.GITHUB_PAGES === "true";
const deploymentHost =
  process.env.VERCEL_PROJECT_PRODUCTION_URL ??
  process.env.VERCEL_URL ??
  "oguzbay-del.github.io";
const site = process.env.SITE_URL ?? `https://${deploymentHost}`;

export default defineConfig({
  site,
  base: isGitHubPages ? "/brewery" : "/",
  trailingSlash: "always",
  output: "static",
  build: {
    format: "directory",
  },
});
