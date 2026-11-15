# GRISTORA domain and production launch setup

**Owner update (15 November 2026):** the owner confirms that `gristora.com` has been purchased. This is an owner-provided fact, not independent registrar verification. Do not describe DNS, ownership verification, HTTPS or production launch as complete until each is checked.

## Intended canonical origin

- Primary: `https://gristora.com`
- Preferred canonical host: apex `gristora.com`
- Secondary: `www.gristora.com` should redirect to the apex host once both are configured.
- Keep the public marketing site separate from the product application; do not invent an app subdomain or sign-in route.

## Current hosting path

The repository already publishes a static preview with GitHub Pages. Before switching the site to the purchased domain:

1. In the repository, open **Settings → Pages** and add `gristora.com` as the custom domain. GitHub recommends configuring the custom domain in Pages before changing DNS, and verifying the domain to prevent takeover.
2. At the registrar/DNS provider, configure the apex records for GitHub Pages:

   | Type | Host | Value |
   |---|---|---|
   | A | @ | 185.199.108.153 |
   | A | @ | 185.199.109.153 |
   | A | @ | 185.199.110.153 |
   | A | @ | 185.199.111.153 |
   | CNAME | www | oguzbay-del.github.io |

   Optional IPv6 AAAA records should only be added if the provider supports them correctly; use GitHub's current official values. Remove conflicting apex A/AAAA/ALIAS/ANAME records. Do not use wildcard DNS.
3. Verify the domain in GitHub account Pages settings using the TXT record GitHub provides. Keep that TXT record after verification.
4. Wait for DNS propagation and GitHub Pages certificate provisioning. Enable **Enforce HTTPS** only when GitHub reports the certificate is ready.
5. Confirm both `https://gristora.com` and `https://www.gristora.com`, redirect behavior, certificate, canonical URLs, sitemap and robots.txt.

Official references:
- https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/managing-a-custom-domain-for-your-github-pages-site
- https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/verifying-your-custom-domain-for-github-pages
- https://docs.github.com/en/pages/getting-started-with-github-pages/securing-your-github-pages-site-with-https

## Build configuration

- The existing GitHub Pages preview retains `/brewery/` as its base path.
- The production build uses `SITE_URL=https://gristora.com` and `SITE_BASE=/`.
- Do not change the preview workflow to production mode until the owner has completed Pages custom-domain setup and DNS/HTTPS checks. The existing preview and production site share the repository's GitHub Pages deployment target.
- Astro reads `SITE_URL` and `SITE_BASE`; sitemap and robots origin derive from the Astro site configuration.

## Launch acceptance

- [ ] Owner confirms registrar access and DNS provider.
- [ ] Custom domain added in repository Settings → Pages.
- [ ] GitHub Pages domain verification TXT record published and verified.
- [ ] Apex A records and www CNAME resolve correctly; no conflicting records.
- [ ] HTTPS certificate ready and Enforce HTTPS enabled.
- [ ] Production build uses `SITE_URL=https://gristora.com`, `SITE_BASE=/`.
- [ ] Root homepage, all routes, 404, canonical metadata, Open Graph, sitemap and robots checked on the custom domain.
- [ ] Owner gives explicit production-launch approval.

**Important:** Domain purchase does not equal trademark clearance. Keep the separate trademark counsel gate open. Do not publish a legal-clearance claim.
