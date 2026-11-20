import type { APIRoute } from "astro";
import { features } from "../data/features";
import { featuresTr } from "../data/features.tr";

const routes = [
  "",
  "homebrewers/",
  "breweries/",
  "how-it-works/",
  "technical-principles/",
  "faq/",
  "pricing/",
  "get-started/",
  "compare/",
  "modules/",
  "features/",
  ...features.map((feature) => `features/${feature.slug}/`),
];
const turkishRoutes = ["tr/", "tr/homebrewers/", "tr/breweries/", "tr/how-it-works/", "tr/technical-principles/", "tr/faq/", "tr/pricing/", "tr/get-started/", "tr/compare/", "tr/modules/", "tr/features/", ...featuresTr.map((feature) => "tr/features/" + feature.slug + "/")];

export const GET: APIRoute = ({ site }) => {
  const origin = new URL(import.meta.env.BASE_URL, site ?? new URL("https://oguzbay-del.github.io"));
  const isCanonicalProduction = origin.hostname === "gristora.com";
  const publishedRoutes = isCanonicalProduction ? [...routes, ...turkishRoutes] : routes;
  const urls = publishedRoutes
    .map((route) => `  <url><loc>${new URL(route, origin).href}</loc></url>`)
    .join("\n");

  return new Response(
    `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urls}\n</urlset>\n`,
    { headers: { "Content-Type": "application/xml; charset=utf-8" } },
  );
};
