# GRISTORA domain and production hosting

**Current hosting:** Cloudflare Pages project `gristora`. The owner has connected `gristora.com` to the Cloudflare-hosted marketing website. This file records the current setup and safe operating rules; it is not a request to change DNS.

## Source and build

- GitHub repository: `oguzbay-del/brewery`
- Website root directory: `marketing-site`
- Framework: Astro, static output
- Install command: `npm install`
- Build command: `npm run build`
- Build output directory: `dist`
- Canonical origin: `https://gristora.com`
- Cloudflare Pages project: `gristora`

Cloudflare Pages provides branch preview deployments. Preview builds use `CF_PAGES_URL` as the canonical origin and are generated with `noindex, nofollow`. The production branch resolves to the canonical `gristora.com` host and is generated with `index, follow`. The production sitemap includes both English and Turkish routes; preview sitemaps exclude the Turkish production-only routes.

## Domain and DNS safety

- Keep DNS and custom-domain configuration in Cloudflare.
- Do not copy DNS records from old hosting providers or change records during normal website development.
- Do not alter the apex or `www` records, redirect rules, SSL/TLS mode, or nameservers without an explicit owner request.
- Confirm the custom-domain status and HTTPS certificate in Cloudflare before recording a hosting change as complete.
- The marketing site remains separate from the GRISTORA product application. Do not invent an app or sign-in subdomain.

## Deployment rules

1. Work on a feature branch and open a pull request.
2. Let GitHub marketing CI run Astro/TypeScript checks, photo optimization, static build, generated-site validation, responsive Playwright/axe checks and Lighthouse.
3. Review the Cloudflare branch preview and its robots/canonical metadata before merging.
4. Merge only after required checks pass.
5. Verify the production deployment after a reviewed merge. Do not make DNS changes as part of the merge.
6. Keep GitHub Pages as a separate `noindex` fallback only.

## Vercel retirement

Vercel is not used for the GRISTORA marketing website. The project lookup in the connected `demleme` Vercel team returned no matching `brewery` project. Do not create a replacement Vercel project, add Vercel environment variables, or rely on `VERCEL_URL` for canonical metadata.

## Launch and SEO checks

- [ ] Cloudflare Pages production deployment is successful.
- [ ] `https://gristora.com/` and `https://gristora.com/tr/` return the expected site.
- [ ] Production pages emit `index, follow`; branch previews emit `noindex, nofollow`.
- [ ] `/robots.txt` allows crawling and points to the production sitemap.
- [ ] `/sitemap.xml` contains canonical English and Turkish production routes.
- [ ] Canonical and Open Graph URLs use `https://gristora.com` in production.
- [ ] HTTPS is active for the custom domain.
- [ ] Owner confirms public production launch.

Do not claim trademark clearance based on domain ownership or hosting configuration.
