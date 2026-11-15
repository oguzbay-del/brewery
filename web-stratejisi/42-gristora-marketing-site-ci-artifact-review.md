# GRISTORA — Marketing-site CI artifact review

**Review date:** 15 November 2026  
**Workflow:** Marketing Site Checks, run `43717341183` (#219)  
**Branch tested:** `marketing/gristora-preferred-logo-direction`  
**Head SHA tested:** `c89d6d26b9ae2f2bce40143bfd5cbaee67a1cf23`  
**Important:** this run tested the existing Demleme-branded public site and documentation changes. It is not a GRISTORA rebrand visual approval.

## 1. Automated result

The single CI build job completed successfully:
- Dependency installation
- Astro and TypeScript checks
- Static site build
- Generated routes, metadata and links validation
- Social preview PNG generation
- Chromium installation
- Responsive browser QA
- Responsive QA screenshot artifact upload

Artifacts:
- `marketing-responsive-qa-screenshots` — GitHub artifact ID `14163530043`, 25 screenshots, 11.6 MB.
- `marketing-social-card-png` — GitHub artifact ID `14163006962`, generated social preview PNG.

Both artifacts were still available at review time and are due to expire on 29 November 2026. The workflow artifact list is the source of truth for availability.

## 2. Manual visual spot-check

Reviewed:
- Homepage full-page screenshot at 1440 px.
- Homepage full-page screenshot at 390 px.
- Professional Breweries full-page screenshot at 390 px.
- Generated 1200 × 630 social-card PNG.

Observations:
- No obvious clipping or horizontal-overflow issue in the three spot-checked page captures; automated responsive QA passed across the full screenshot set.
- Homepage hierarchy retains Professional Breweries as the primary path and Homebrewers as a separate secondary path.
- The mobile homepage stacks hero, audience cards, technical principles, feature cards and CTA vertically.
- The current social card is legible and correctly generated at 1200 × 630, but still carries the legacy Demleme wordmark and an abstract brewing-vessel illustration.
- Current visual assets are not GRISTORA assets and must not be treated as the approved selected logo.

This is a visual spot-check, not a complete manual audit of all 25 screenshots, keyboard navigation, screen-reader output, 200% zoom or physical devices.

## 3. Rebrand implications

When WP-17 is authorised:
1. Replace the social-card wordmark with the approved GRISTORA master.
2. Reassess the illustration against the approved logo direction; do not automatically carry forward the current vessel graphic if it conflicts with the selected editorial/technical identity.
3. Regenerate the social-card PNG from its SVG source and rerun metadata/asset validation.
4. Review new screenshots at desktop, tablet and mobile sizes; keep manual keyboard, screen-reader, zoom and real-device checks as release gates.
5. Do not claim that this existing Demleme visual review approves the GRISTORA brand.

## 4. Deployment distinction

GitHub Actions CI success is separate from Vercel deployment. The Vercel preview was blocked by the existing 24-hour Hobby deployment quota. No live GRISTORA site or successful preview deployment is evidenced by this run.
