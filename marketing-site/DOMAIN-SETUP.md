# GRISTORA domain and production launch setup

**Owner update (15 November 2026):** the owner confirms that `gristora.com` has been purchased. This is an owner-provided fact, not independent registrar verification. DNS, domain verification, HTTPS and production launch are separate checks and must not be described as complete until verified.

## Hosting decision

Use a **dedicated Vercel project for the public marketing website**. Keep GitHub as the source repository and CI/review system.

- Repository: `oguzbay-del/brewery`
- Vercel project name: `gristora-marketing` (suggested; create as a separate project)
- Vercel Root Directory: `marketing-site`
- Framework preset: Astro
- Build command: `npm run build`
- Output directory: `dist`
- Install command: `npm install`
- Production plan: Vercel Pro before operating the commercial marketing site publicly. Hobby is for personal, non-commercial use.
- Do not attach this website to or modify the existing product Vercel project. The marketing site must have its own project and deployment settings.

The site is statically generated. Do not add an Astro server adapter.

## Intended canonical origin

- Primary: `https://gristora.com`
- Preferred canonical host: apex `gristora.com`
- Secondary: `www.gristora.com` should redirect to the apex host.
- Keep the public marketing site separate from the product application. Do not invent an app subdomain or sign-in route.

## Vercel project setup

1. In Vercel, create a **new project** from `oguzbay-del/brewery`; do not reuse the existing product project.
2. Set Root Directory to `marketing-site` and confirm the Astro build settings above.
3. Keep Preview deployments enabled for pull requests. Do not set a production domain on the product project.
4. Set Production Environment Variable `SITE_URL=https://gristora.com`. Leave `SITE_BASE` unset (root path).
5. Preview builds use Vercel's `VERCEL_URL` automatically for their own canonical origin. GitHub Pages fallback continues to use `/brewery/`.
6. Upgrade the dedicated marketing project to Vercel Pro before public commercial operation.

## DNS and domain connection

1. In the **dedicated marketing Vercel project**, open Settings → Domains and add both `gristora.com` and `www.gristora.com`.
2. Copy the exact DNS records Vercel displays for this project. DNS targets can vary by account/project; do not blindly reuse old GitHub Pages records or stale generic values.
3. At the registrar/DNS provider, publish Vercel's requested apex and www records. Remove conflicting records only after confirming they belong to the previous hosting target. Do not use wildcard DNS unless explicitly required.
4. If Vercel requests domain ownership verification, publish its exact TXT record and retain it as instructed.
5. Configure `www.gristora.com` to redirect to `https://gristora.com` in the Vercel project.
6. Wait for Vercel to report the domain as configured and HTTPS certificate as issued. Do not claim HTTPS is ready before that status is visible.

## Build and URL behavior

- Production: `SITE_URL=https://gristora.com`, `SITE_BASE=/`.
- Vercel Preview: Astro uses `VERCEL_URL` as the preview origin unless `SITE_URL` is explicitly overridden.
- GitHub Pages fallback: `GITHUB_PAGES=true` retains `https://oguzbay-del.github.io/brewery/`.
- Sitemap and robots origin derive from the active Astro site configuration.
- Never change the existing GitHub Pages fallback into a production deployment as part of this Vercel migration.

## Launch acceptance

- [ ] Dedicated Vercel project created with Root Directory `marketing-site`.
- [ ] Project is separate from the product Vercel project.
- [ ] Vercel Pro enabled before commercial public operation.
- [ ] Production environment variable `SITE_URL=https://gristora.com` configured.
- [ ] Preview deployment uses its own Vercel URL and correct canonical metadata.
- [ ] Both apex and www domains added to the marketing project.
- [ ] DNS records match the values displayed by Vercel and resolve correctly.
- [ ] Domain verification complete if Vercel requests it.
- [ ] HTTPS certificate issued and active.
- [ ] www-to-apex redirect verified.
- [ ] Homepage, all routes, 404, canonical metadata, Open Graph, sitemap and robots checked on the custom domain.
- [ ] Owner gives explicit production-launch approval.

**Important:** Domain purchase does not equal trademark clearance. Keep the separate trademark counsel gate open. Do not publish a legal-clearance claim.
