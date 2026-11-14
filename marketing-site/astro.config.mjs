import { defineConfig } from "astro/config";

export default defineConfig({
  site: "https://oguzbay-del.github.io",
  base: "/brewery",
  trailingSlash: "always",
  output: "static",
  build: {
    format: "directory",
  },
});
