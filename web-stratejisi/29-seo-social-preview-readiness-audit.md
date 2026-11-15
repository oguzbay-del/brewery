# Marketing SEO and Social Preview Readiness Audit

**Status:** Source-level audit; no launch or platform-rendering claim.  
**Reviewed:** 15 November 2026

## Findings

### Page titles and descriptions
- Core audience and explanatory pages have distinct, descriptive titles and page-specific descriptions.
- The recent SEO pass is in place for Homebrewers, Breweries, How It Works, Technical Principles and FAQ.
- Structured-data names should match visible page purpose and canonical URL. JSON-LD describes marketing pages, not product availability.
- Do not add unsupported feature, accuracy, integration, customer or launch claims.

### Social title consistency
Source review found two small inconsistencies:
- technical-principles.astro uses a shorter Twitter title while og:title uses the descriptive page title.
- how-it-works.astro uses a shorter Twitter title while og:title uses the descriptive page title.

Use the same approved page title for HTML title, Open Graph title, Twitter title and WebPage structured-data name unless a channel-specific variant is deliberate and documented.

### Social image readiness
- Reviewed page heads include og:type, og:site_name, og:title, og:description, og:url and twitter:card.
- They do not yet declare og:image / twitter:image.
- Existing SVG illustrations are not automatically validated social preview assets. Before wiring metadata, create a dedicated branded raster card (prefer 1200 × 630 PNG), then validate its exact public URL, dimensions, response/content type, safe text margins and appearance in target platform debuggers.
- Do not assume an SVG URL is universally supported by social crawlers; support and crawler behaviour vary.
- A single generic card can be used initially if it communicates the brand/category without implying availability.

### Canonical and crawler checks
Before public release, verify on the final domain:
- canonical resolves to intended HTTPS origin; og:url matches canonical;
- og:image and twitter:image resolve anonymously with HTTP 200 and image content type;
- robots.txt and sitemap.xml reference final origin;
- no staging or GitHub Pages URL leaks into production metadata;
- structured data parses and matches visible content.

## Recommended implementation order
1. Correct the two Twitter title mismatches.
2. Design/export the dedicated raster social card; retain documented source SVG if useful.
3. Add shared social-image metadata only after final image path and public origin are confirmed.
4. Run Astro check/build and existing Playwright route/metadata tests.
5. Inspect the rendered card with platform-specific preview tools once the public URL is stable.

## Explicitly not done
- No domain, Vercel project, production hosting or DNS setting changed.
- No analytics, tracking, consent or social pixel added.
- No production publishing or platform compatibility certification.
- No claim that a crawler preview has been visually checked.
