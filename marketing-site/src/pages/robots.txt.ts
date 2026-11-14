import type { APIRoute } from "astro";

export const GET: APIRoute = ({ site }) => {
  const origin = new URL(import.meta.env.BASE_URL, site ?? new URL("https://oguzbay-del.github.io"));
  const body = `User-agent: *\nAllow: /\n\nSitemap: ${new URL("sitemap.xml", origin).href}\n`;

  return new Response(body, {
    headers: { "Content-Type": "text/plain; charset=utf-8" },
  });
};
