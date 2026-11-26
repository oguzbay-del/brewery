# GRISTORA domain and production hosting

**Current hosting:** Cloudflare Pages project `gristora`. The canonical production hostname is `https://gristora.com`; `www.gristora.com` should redirect to the apex. The owner applies DNS and dashboard changes. This guide is an execution checklist, not authorization to change DNS.

## Source and build

- GitHub repository: `oguzbay-del/brewery`
- Website root directory: `marketing-site`
- Framework: Astro, static output
- Install command: `npm install`
- Build command: `npm run build`
- Build output directory: `dist`
- Canonical origin: `https://gristora.com`
- Cloudflare Pages project: `gristora`
- Default Pages hostname: `gristora.pages.dev`

Cloudflare Pages provides branch preview deployments. Preview builds use `CF_PAGES_URL` as the canonical origin and are generated with `noindex, nofollow`. The production branch resolves to the canonical `gristora.com` host and is generated with `index, follow`. The production sitemap includes English and Turkish routes; preview sitemaps exclude the Turkish production-only routes.

## Intended DNS records

The records below describe the intended final configuration. **Do not add, edit, delete or replace records as part of a code change.** First inspect the live Cloudflare DNS zone and the Pages project's Custom domains screen. If the dashboard already shows equivalent active records, leave them unchanged.

| Type | Name | Target / content | Proxy |
|---|---|---|---|
| CNAME | `@` (apex) | `gristora.pages.dev` | Proxied |
| A | `www` | `192.0.2.1` | Proxied |

Notes:

- The apex CNAME uses Cloudflare's CNAME flattening. For a Pages custom apex domain, the domain must be a zone on the same Cloudflare account as the Pages project, and the nameservers must already point to Cloudflare. Cloudflare Pages may create the apex CNAME automatically after the custom domain is attached.
- The `www` A record uses the reserved placeholder address documented by Cloudflare for a proxied, originless redirect hostname. It is not an origin server IP. Keep it proxied so the redirect rule can run.
- Do not assume the records are currently absent or overwrite existing records. The live zone and Pages dashboard are authoritative.
- If the Pages dashboard proposes a different record or reports a domain verification issue, stop and follow the dashboard's current instructions rather than improvising.

## Owner checklist: connect and verify the custom domain

1. Open Cloudflare Dashboard → **Workers & Pages** → Pages project **gristora** → **Custom domains**.
2. Confirm that `gristora.com` is listed and active. If it is not listed, choose **Set up a domain**, enter `gristora.com`, and follow the dashboard flow. Do not proceed by creating a DNS record alone: Pages requires the hostname to be associated with the project.
3. Confirm that the `gristora.com` zone is in the same Cloudflare account as the Pages project and that its nameservers are already Cloudflare nameservers. Do not change nameservers without a separate owner decision.
4. In **DNS → Records**, inspect the apex record. The intended target is `gristora.pages.dev` using a proxied CNAME (Cloudflare flattens the apex CNAME). If Pages created the record, do not recreate it.
5. In **Custom domains**, also check whether `www.gristora.com` is already attached. The intended behavior is a permanent redirect from `www` to `https://gristora.com`; do not attach or detach it without checking the existing setup first.
6. If `www` is not already configured, create/confirm the proxied `A` record shown above, then create a **Bulk Redirect** from `www.gristora.com` to `https://gristora.com` with status **301**. Enable query-string preservation, subpath matching, path-suffix preservation and subdomain inclusion. Do not create a second competing redirect if an equivalent rule already exists.
7. In **SSL/TLS**, confirm the Pages custom hostname has an active certificate and HTTPS works. Do not change the zone's SSL/TLS mode as a troubleshooting guess.
8. Check any existing CAA records if certificate issuance is blocked. Cloudflare documents that restrictive CAA records can prevent Pages from issuing a certificate; only change CAA after reviewing the current zone and the required certificate authorities.
9. Record the final DNS values, proxy state, Pages custom-domain status, redirect rule name and certificate status in the issue. Do not record secrets or account identifiers.

## Verification commands

Run these after the owner has completed the dashboard/DNS steps. They are read-only checks:

```sh
curl -sSIL https://gristora.com/
curl -sSIL https://www.gristora.com/
curl -sSIL https://gristora.com/tr/
curl -sS https://gristora.com/robots.txt
curl -sS https://gristora.com/sitemap.xml
```

Expected results:

- `https://gristora.com/` and `https://gristora.com/tr/` return successful HTTPS responses.
- `https://www.gristora.com/` returns a permanent redirect to the matching path on `https://gristora.com`; query strings are preserved.
- The production pages emit `index, follow`; branch previews emit `noindex, nofollow`.
- `robots.txt` permits the intended production crawl and points to the production sitemap.
- The sitemap contains canonical English and Turkish production routes.
- Canonical and Open Graph URLs use `https://gristora.com` in production.
- The HTTPS certificate is valid for both hostnames.

## Deployment rules

1. Work on a feature branch and open a pull request.
2. Let GitHub marketing CI run Astro/TypeScript checks, photo optimization, static build, generated-site validation, responsive Playwright/axe checks and Lighthouse.
3. Review the Cloudflare branch preview and its robots/canonical metadata before merging.
4. Merge only after required checks pass.
5. Verify the production deployment after a reviewed merge. Do not make DNS changes as part of the merge.
6. Keep GitHub Pages as a separate `noindex` fallback only.

## Vercel retirement

Vercel is not used for the GRISTORA marketing website. Do not create a replacement Vercel project, add Vercel environment variables, or rely on `VERCEL_URL` for canonical metadata.

## Launch and SEO checks

- [ ] Cloudflare Pages production deployment is successful.
- [ ] `https://gristora.com/` and `https://gristora.com/tr/` return the expected site.
- [ ] `www.gristora.com` redirects to the apex with HTTP 301 and preserves paths and query strings.
- [ ] Production pages emit `index, follow`; branch previews emit `noindex, nofollow`.
- [ ] `/robots.txt` allows crawling and points to the production sitemap.
- [ ] `/sitemap.xml` contains canonical English and Turkish production routes.
- [ ] Canonical and Open Graph URLs use `https://gristora.com` in production.
- [ ] HTTPS is active for the apex and `www` hostnames.
- [ ] Owner confirms public production launch.

Do not claim trademark clearance based on domain ownership or hosting configuration.

## Official Cloudflare references

- [Pages custom domains](https://developers.cloudflare.com/pages/configuration/custom-domains/) — attach the hostname to the Pages project; apex domains require the zone and Cloudflare nameservers.
- [CNAME flattening](https://developers.cloudflare.com/dns/cname-flattening/) — supports a CNAME at the zone apex.
- [Redirecting www to the apex](https://developers.cloudflare.com/pages/how-to/www-redirect/) — proxied placeholder A record and Bulk Redirect configuration.
- [Pages custom-domain troubleshooting](https://developers.cloudflare.com/pages/configuration/debugging-pages/) — certificate validation and CAA troubleshooting.
