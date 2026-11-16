# GRISTORA — Stitch overnight brief (marketing site remainder)

Owner: paste the batches below into Stitch **in order**, each into the same Stitch project ("GRISTORA Marketing Website Implementation"). Each batch is one Stitch prompt. Do not paste the whole file at once: Stitch makes a few screens per prompt and truncates very long prompts. When a batch finishes, paste the next one. If Stitch asks a question, answer "Follow the GLOBAL RULES and use the given copy exactly".

Batches: 1 = mobile versions of existing pages, 2–9 = 16 feature detail pages (2 per batch), 10 = feature directory + 404, 11 = final consistency pass.

Next morning: tell Claude "Stitch bitti, port et". Claude reads the project through the Stitch MCP, a Sonnet subagent ports it to Astro, and the guardrail checks (document 41) remove any forbidden copy.

Security: the API key must not be written in this file or in the repository.

---

## Batch 1 — mobile versions of existing pages

```text
GLOBAL RULES (apply to every screen in this batch):
- Brand: GRISTORA, descriptor "Brewing Intelligence". Brewing software for professional breweries, with a separate Homebrewers path. English (US/UK audience).
- Visual system (already approved, keep it): page #F7F6F3, deep forest #0B2F26 (text and dark bands), white cards #FFFFFF, muted text #475569, soft line rgba(11,47,38,.18), amber fill #F4A261 for pill CTAs and accents (text on amber is #0B2F26), amber text tone #8B4A1F on light backgrounds, olive #7A8B75 for minor accents, band #F0EFEB. Headlines Plus Jakarta Sans, body Inter. Pill navigation, pill buttons, card radius 18px, panel radius 24px, soft shadow. Same header and footer as the existing GRISTORA pages: wordmark left; nav Product, Breweries, Homebrewers, Pricing, How it works, Technical principles, FAQ; footer "© 2026 GRISTORA · Made for the craft of brewing."
- Copy: use ONLY the copy given in the prompt, word for word. You may shorten nothing and add nothing except navigation labels and the short UI labels listed.
- FORBIDDEN (never generate): testimonials, quotes from customers, customer logos, star ratings, user counts, metrics or percentages, prices or "free" offers, "start trial", "sign up", "book a demo", "live", "available now", "production-ready", "AI", "predictive", "machine learning", forecasts, telemetry dashboards with live numbers, IoT/SCADA/PLC claims, named hardware brands (Tilt, iSpindel, Plaato, etc.), certifications (SOC 2, ISO, GDPR-compliant), uptime/SLA, fake product screenshots with realistic data.
- Illustrations: abstract editorial shapes only (grain, vessel silhouettes, lines). If you show a UI-like panel, label it "Concept" and use placeholder bars, not numbers.
- Accessibility: WCAG 2.1 AA contrast, visible focus, 44px touch targets, one H1 per page, no text in images.
- Output for each screen: a desktop version (1280px wide) AND a mobile version (390px wide). Mobile: hamburger menu, single column, no horizontal scroll, sticky header allowed.

TASK: For every existing desktop screen in this project (Home, Breweries, Homebrewers, Pricing, How it works, Product, FAQ, Technical principles, Compare, Get started), generate the 390px mobile version. Keep the same copy, the same section order and the same visual language. Stack multi-column sections into one column. Dark forest bands stay dark. Tables (Compare, Pricing) become stacked cards with the row label as the card heading. FAQ uses accordions. The Pricing page must not show any price number: keep the text exactly as in the desktop screen.
```

---

## Batch 2 — recipe-design, brewing-calculations

```text
GLOBAL RULES (apply to every screen in this batch):
- Brand: GRISTORA, descriptor "Brewing Intelligence". Brewing software for professional breweries, with a separate Homebrewers path. English (US/UK audience).
- Visual system (already approved, keep it): page #F7F6F3, deep forest #0B2F26 (text and dark bands), white cards #FFFFFF, muted text #475569, soft line rgba(11,47,38,.18), amber fill #F4A261 for pill CTAs and accents (text on amber is #0B2F26), amber text tone #8B4A1F on light backgrounds, olive #7A8B75 for minor accents, band #F0EFEB. Headlines Plus Jakarta Sans, body Inter. Pill navigation, pill buttons, card radius 18px, panel radius 24px, soft shadow. Same header and footer as the existing GRISTORA pages: wordmark left; nav Product, Breweries, Homebrewers, Pricing, How it works, Technical principles, FAQ; footer "© 2026 GRISTORA · Made for the craft of brewing."
- Copy: use ONLY the copy given in the prompt, word for word. You may shorten nothing and add nothing except navigation labels and the short UI labels listed.
- FORBIDDEN (never generate): testimonials, quotes from customers, customer logos, star ratings, user counts, metrics or percentages, prices or "free" offers, "start trial", "sign up", "book a demo", "live", "available now", "production-ready", "AI", "predictive", "machine learning", forecasts, telemetry dashboards with live numbers, IoT/SCADA/PLC claims, named hardware brands (Tilt, iSpindel, Plaato, etc.), certifications (SOC 2, ISO, GDPR-compliant), uptime/SLA, fake product screenshots with realistic data.
- Illustrations: abstract editorial shapes only (grain, vessel silhouettes, lines). If you show a UI-like panel, label it "Concept" and use placeholder bars, not numbers.
- Accessibility: WCAG 2.1 AA contrast, visible focus, 44px touch targets, one H1 per page, no text in images.
- Output for each screen: a desktop version (1280px wide) AND a mobile version (390px wide). Mobile: hamburger menu, single column, no horizontal scroll, sticky header allowed.

TASK: Generate the following 2 feature detail pages. All 16 feature pages share ONE template; keep it identical to earlier feature pages in this project (if this is the first, define it cleanly so later pages reuse it).

SCREEN: Feature detail page "/features/recipe-design/"
Layout: breadcrumb (Home › Product › Recipe design & management); hero with eyebrow, H1, summary; two pill tags rows ("For": audiences, "Roadmap stage": stages) styled as neutral chips (not as badges of availability); then one card per section in a 2-column grid on desktop (1 column mobile), each card with heading, intro line and a checklist; then a dark forest band with "Explore all features" (link to /features/) and "How it works" (link to /how-it-works/); then footer with "All features ↑".
COPY (use exactly):
Eyebrow: RECIPES
H1: Recipe design & management
Summary: From a guided first recipe to a deeply configurable professional formulation, GRISTORA keeps recipes understandable from version to version. Recipes stay understandable from version to version.
For: Breweries, Homebrewers
Roadmap stage: Homebrew MVP, Business core

Section — Start the way that suits you
Intro: Four entry points lead into one consistent recipe workflow.
- Start from a blank recipe with style- and target-led ingredient suggestions
- Use style-based starter templates and adapt them to your equipment and batch size
- Copy an existing recipe or create a deliberate new version
- Import BeerXML, BeerJSON, BeerSmith .bsmx, Brewfather and Brewer's Friend exports with ingredient, equipment, style and unit matching

Section — A recipe editor built for iteration
Intro: The editor is designed to make changes and their consequences visible.
- Edit fermentables, hops, yeast, miscellaneous ingredients, mash, boil and fermentation details
- Adjust grain percentages and target OG, IBU or colour while keeping selected values in balance
- Convert all-grain recipes to extract and back; substitute ingredients and round quantities
- Scale by batch volume, brewhouse efficiency or equipment while recalculating losses
- Review recipe checks, warnings and one-step correction suggestions

Section — Styles, versions and collaboration
Intro: Keep the reference and history behind a recipe clear.
- BJCP 2021, BJCP mead/cider and Brewers Association style guides, with custom guide support
- Compare style ranges and map a recipe between guides
- Compare two recipes or versions side by side
- Keep immutable published versions; business recipes move through draft, review and approval
- Lock a recipe against accidental edits and preserve the version used by each batch

Section — Recipe library and supported brew types
Intro: Keep a growing recipe collection organised and useful.
- Organise recipes and batches with search, filters, folders and labels
- Pin favourites and choose which columns appear in recipe lists
- Support beer, mead, cider and seltzer recipes, including all-grain, extract, partial mash and BIAB approaches
- Keep recipe notes, colour choices, images and custom fields
- Archive and restore recipe history without silently overwriting published versions


=====

SCREEN: Feature detail page "/features/brewing-calculations/"
Layout: breadcrumb (Home › Product › Brewing calculations); hero with eyebrow, H1, summary; two pill tags rows ("For": audiences, "Roadmap stage": stages) styled as neutral chips (not as badges of availability); then one card per section in a 2-column grid on desktop (1 column mobile), each card with heading, intro line and a checklist; then a dark forest band with "Explore all features" (link to /features/) and "How it works" (link to /how-it-works/); then footer with "All features ↑".
COPY (use exactly):
Eyebrow: THE CALCULATION ENGINE
H1: Brewing calculations
Summary: A shared, server-side calculation engine serves the web and Apple apps, with clear inputs, consistent units and an explanation behind every result.
For: Breweries, Homebrewers
Roadmap stage: Homebrew MVP, Business core, Business second release, Under consideration

Section — Core recipe calculations
Intro: A broad calculation set supports formulation and review.
- Original and final gravity (OG / FG), apparent attenuation and alcohol by volume (ABV)
- Bitterness (IBU), including Tinseth as the default, late additions, whirlpool and hop-stand behaviour
- Colour estimates with selectable calculation approach
- Mash-temperature and yeast-tolerance adjustments to fermentation estimates
- Batch, kettle, trub and volume-chain calculations

Section — Advanced brewing models
Intro: The engine includes specialist methods and configurable assumptions.
- Hop-ageing and alpha-acid adjustments using supplied inputs
- Starter growth and pitching calculations
- Carbonation and priming for bottles and kegs, including keg headspace and pressure limits
- Mead and wine fermentation calculations, with corrected YAN handling
- Extract, partial-boil, dilution and no-chill calculation tools

Section — Explainable results
Intro: Technical transparency is a product principle.
- A “How it was calculated” view for key results
- Calculation engine version stored with each saved result
- Consistent SI-based internal values with regional display and unit conversion
- Reference test vectors and golden-recipe validation for every engine release

Section — Specialist methods
Intro: Additional methods sit apart from the core calculation set.
- No-chill IBU, hop-oil ratios, BU:GU and alternative IBU and colour methods
- Fruit additions, pH temperature correction and total-acidity adjustment for cider and wine
- Sour and mixed fermentation targets and barrel-ageing calculations
- Keg line-balance tools for hose diameter, length and serving pressure
- Method selection and advanced defaults in calculation settings

```

---

## Batch 3 — water-mash-yeast, brew-day

```text
GLOBAL RULES (apply to every screen in this batch):
- Brand: GRISTORA, descriptor "Brewing Intelligence". Brewing software for professional breweries, with a separate Homebrewers path. English (US/UK audience).
- Visual system (already approved, keep it): page #F7F6F3, deep forest #0B2F26 (text and dark bands), white cards #FFFFFF, muted text #475569, soft line rgba(11,47,38,.18), amber fill #F4A261 for pill CTAs and accents (text on amber is #0B2F26), amber text tone #8B4A1F on light backgrounds, olive #7A8B75 for minor accents, band #F0EFEB. Headlines Plus Jakarta Sans, body Inter. Pill navigation, pill buttons, card radius 18px, panel radius 24px, soft shadow. Same header and footer as the existing GRISTORA pages: wordmark left; nav Product, Breweries, Homebrewers, Pricing, How it works, Technical principles, FAQ; footer "© 2026 GRISTORA · Made for the craft of brewing."
- Copy: use ONLY the copy given in the prompt, word for word. You may shorten nothing and add nothing except navigation labels and the short UI labels listed.
- FORBIDDEN (never generate): testimonials, quotes from customers, customer logos, star ratings, user counts, metrics or percentages, prices or "free" offers, "start trial", "sign up", "book a demo", "live", "available now", "production-ready", "AI", "predictive", "machine learning", forecasts, telemetry dashboards with live numbers, IoT/SCADA/PLC claims, named hardware brands (Tilt, iSpindel, Plaato, etc.), certifications (SOC 2, ISO, GDPR-compliant), uptime/SLA, fake product screenshots with realistic data.
- Illustrations: abstract editorial shapes only (grain, vessel silhouettes, lines). If you show a UI-like panel, label it "Concept" and use placeholder bars, not numbers.
- Accessibility: WCAG 2.1 AA contrast, visible focus, 44px touch targets, one H1 per page, no text in images.
- Output for each screen: a desktop version (1280px wide) AND a mobile version (390px wide). Mobile: hamburger menu, single column, no horizontal scroll, sticky header allowed.

TASK: Generate the following 2 feature detail pages. All 16 feature pages share ONE template; keep it identical to earlier feature pages in this project (if this is the first, define it cleanly so later pages reuse it).

SCREEN: Feature detail page "/features/water-mash-yeast/"
Layout: breadcrumb (Home › Product › Water, mash & yeast); hero with eyebrow, H1, summary; two pill tags rows ("For": audiences, "Roadmap stage": stages) styled as neutral chips (not as badges of availability); then one card per section in a 2-column grid on desktop (1 column mobile), each card with heading, intro line and a checklist; then a dark forest band with "Explore all features" (link to /features/) and "How it works" (link to /how-it-works/); then footer with "All features ↑".
COPY (use exactly):
Eyebrow: PROCESS DESIGN
H1: Water, mash & yeast
Summary: Plan water chemistry, mash steps and yeast preparation as connected parts of the recipe rather than isolated calculators.
For: Breweries, Homebrewers
Roadmap stage: Homebrew MVP, Under consideration

Section — Water profiles
Intro: Build a water profile from source water and a target style or recipe.
- Save source-water and target-water profiles
- Enter a laboratory water report and manage ion concentrations
- Calculate salt additions and acid adjustments
- Review ion balance, hardness and regional hardness units
- Compare water profiles and retain the report used for a recipe

Section — Mash planning
Intro: The mash plan connects ingredient choices, temperatures and volumes.
- Mash schedule with rests, temperatures, durations and infusion steps
- Mash pH estimation using the selected model
- Decoction calculator for mash volume and temperature steps
- Mash water, sparge water and volume calculations
- BIAB-specific equipment and absorption handling

Section — Yeast preparation
Intro: Keep yeast requirements and fermentation expectations visible.
- Yeast selection and age details
- Starter planning, including two-stage starters
- Dry yeast pitching guidance by hectolitre in business mode
- Fermentation profiles and target conditions
- Harvest and generation tracking in the business core; advanced home mode can enable yeast management


=====

SCREEN: Feature detail page "/features/brew-day/"
Layout: breadcrumb (Home › Product › Brew day & production records); hero with eyebrow, H1, summary; two pill tags rows ("For": audiences, "Roadmap stage": stages) styled as neutral chips (not as badges of availability); then one card per section in a 2-column grid on desktop (1 column mobile), each card with heading, intro line and a checklist; then a dark forest band with "Explore all features" (link to /features/) and "How it works" (link to /how-it-works/); then footer with "All features ↑".
COPY (use exactly):
Eyebrow: BREW DAY
H1: Brew day & production records
Summary: A brew-day workspace keeps the recipe plan separate from what actually happened, while remaining usable beside wet equipment and paper records.
For: Breweries, Homebrewers
Roadmap stage: Homebrew MVP, Business core

Section — A guided brew-day timeline
Intro: Move through a clear sequence on phone, tablet or web.
- Full-screen step-by-step iPhone workflow and a detailed iPad/web step list
- Optional timers, start/finish controls and editable actual times
- Next-step guidance, target values and measurement entry
- Apple Live Activity, Dynamic Island and time-sensitive notifications on supported devices
- Multiple batches can run at the same time

Section — Plan versus actual
Intro: The batch records reality without silently changing the source recipe.
- Record actual quantities, time, temperature and volume
- Add, skip, reorder or correct steps; record ingredient substitutions and additions
- Mark deviations and recalculate actual OG, IBU and ABV when online
- Compare measured values with targets and receive contextual interpretation
- Optionally turn the actual outcome into a new recipe draft

Section — Paper-friendly workflow
Intro: Digital and paper workflows are designed to work together.
- Printable A4 or US Letter brew sheets with planned and actual fields
- QR code connects a printed sheet to its batch
- Enter paper measurements in the same numbered sequence
- Scan brew sheets with Apple document scanning and review extracted values side by side
- Business workspaces can require scanned records and apply retention rules
- Customise brew-sheet sections and fields in Home mode
- Use quick measurement entry from reminders and supported system shortcuts

```

---

## Batch 4 — fermentation-monitoring, equipment-calibration

```text
GLOBAL RULES (apply to every screen in this batch):
- Brand: GRISTORA, descriptor "Brewing Intelligence". Brewing software for professional breweries, with a separate Homebrewers path. English (US/UK audience).
- Visual system (already approved, keep it): page #F7F6F3, deep forest #0B2F26 (text and dark bands), white cards #FFFFFF, muted text #475569, soft line rgba(11,47,38,.18), amber fill #F4A261 for pill CTAs and accents (text on amber is #0B2F26), amber text tone #8B4A1F on light backgrounds, olive #7A8B75 for minor accents, band #F0EFEB. Headlines Plus Jakarta Sans, body Inter. Pill navigation, pill buttons, card radius 18px, panel radius 24px, soft shadow. Same header and footer as the existing GRISTORA pages: wordmark left; nav Product, Breweries, Homebrewers, Pricing, How it works, Technical principles, FAQ; footer "© 2026 GRISTORA · Made for the craft of brewing."
- Copy: use ONLY the copy given in the prompt, word for word. You may shorten nothing and add nothing except navigation labels and the short UI labels listed.
- FORBIDDEN (never generate): testimonials, quotes from customers, customer logos, star ratings, user counts, metrics or percentages, prices or "free" offers, "start trial", "sign up", "book a demo", "live", "available now", "production-ready", "AI", "predictive", "machine learning", forecasts, telemetry dashboards with live numbers, IoT/SCADA/PLC claims, named hardware brands (Tilt, iSpindel, Plaato, etc.), certifications (SOC 2, ISO, GDPR-compliant), uptime/SLA, fake product screenshots with realistic data.
- Illustrations: abstract editorial shapes only (grain, vessel silhouettes, lines). If you show a UI-like panel, label it "Concept" and use placeholder bars, not numbers.
- Accessibility: WCAG 2.1 AA contrast, visible focus, 44px touch targets, one H1 per page, no text in images.
- Output for each screen: a desktop version (1280px wide) AND a mobile version (390px wide). Mobile: hamburger menu, single column, no horizontal scroll, sticky header allowed.

TASK: Generate the following 2 feature detail pages. All 16 feature pages share ONE template; keep it identical to earlier feature pages in this project (if this is the first, define it cleanly so later pages reuse it).

SCREEN: Feature detail page "/features/fermentation-monitoring/"
Layout: breadcrumb (Home › Product › Fermentation & batch tracking); hero with eyebrow, H1, summary; two pill tags rows ("For": audiences, "Roadmap stage": stages) styled as neutral chips (not as badges of availability); then one card per section in a 2-column grid on desktop (1 column mobile), each card with heading, intro line and a checklist; then a dark forest band with "Explore all features" (link to /features/) and "How it works" (link to /how-it-works/); then footer with "All features ↑".
COPY (use exactly):
Eyebrow: FERMENTATION
H1: Fermentation & batch tracking
Summary: Follow each batch from brew day through fermentation, conditioning and packaging, using device readings or manual measurements.
For: Breweries, Homebrewers
Roadmap stage: Homebrew MVP, Business core

Section — A complete batch timeline
Intro: Each brew has its own record and stage history.
- Track planned, brewing, fermenting, conditioning, ready-to-package, packaged and closed stages
- Run multiple batches and fermenters concurrently
- Keep measurements, notes, photos and events on the batch timeline
- Compare plan and actual values in the batch summary
- Record tasting notes and use them to inform future recipe work

Section — Measurements and insights
Intro: Bring readings together in one view.
- Gravity, temperature and pH readings from devices or manual entry
- Charts with measurement-source labels for device, manual and paper readings
- Fermentation progress highlights
- Dry-hop, cold-crash and packaging timing suggestions
- Manual-reading reminders when no sensor is connected

Section — Rules and alerts
Intro: Alerts respond to the process and do not create unnecessary noise.
- Rules such as gravity stable for several days or temperature outside a target
- Three priority levels, quiet hours, grouped alerts and daily summaries
- One notification inbox with acknowledge, snooze, ignore and history
- Business shift routing and escalation to the next responsible person
- Quick SG and temperature entry directly from a manual-reading reminder
- Home widgets and tank-focused widgets for at-a-glance fermentation status

Section — Batch comparison and conditioning
Intro: Use completed batches to understand repeatability over time.
- Compare batches made from the same recipe by OG, FG, efficiency and tasting score
- Track bottle and keg conditioning progress towards target carbonation in Home mode
- Turn tasting feedback into a recipe draft in Home mode


=====

SCREEN: Feature detail page "/features/equipment-calibration/"
Layout: breadcrumb (Home › Product › Equipment & calibration); hero with eyebrow, H1, summary; two pill tags rows ("For": audiences, "Roadmap stage": stages) styled as neutral chips (not as badges of availability); then one card per section in a 2-column grid on desktop (1 column mobile), each card with heading, intro line and a checklist; then a dark forest band with "Explore all features" (link to /features/) and "How it works" (link to /how-it-works/); then footer with "All features ↑".
COPY (use exactly):
Eyebrow: EQUIPMENT
H1: Equipment & calibration
Summary: Equipment profiles reflect the real behaviour of a home setup or professional brewhouse, and improve through measured batch outcomes.
For: Breweries, Homebrewers
Roadmap stage: Homebrew MVP, Business core

Section — Equipment profiles
Intro: Set up the equipment that shapes every calculation.
- Guided equipment setup for a first profile
- Mash tun, kettle, fermenter and packaging vessel details
- Volume losses, grain absorption, evaporation, dead space and cooling shrinkage
- Multiple brewhouses and vessel profiles for business workspaces
- Equipment-specific recipe scaling and efficiency assumptions

Section — Vessel readiness
Intro: Professional vessel details connect equipment planning to production.
- Tank capacity, pressure limit and current occupancy
- Cleaning status and last cleaning record
- Vessel availability in the production calendar
- Equipment calibration records

```

---

## Batch 5 — planning-scheduling, inventory-costing

```text
GLOBAL RULES (apply to every screen in this batch):
- Brand: GRISTORA, descriptor "Brewing Intelligence". Brewing software for professional breweries, with a separate Homebrewers path. English (US/UK audience).
- Visual system (already approved, keep it): page #F7F6F3, deep forest #0B2F26 (text and dark bands), white cards #FFFFFF, muted text #475569, soft line rgba(11,47,38,.18), amber fill #F4A261 for pill CTAs and accents (text on amber is #0B2F26), amber text tone #8B4A1F on light backgrounds, olive #7A8B75 for minor accents, band #F0EFEB. Headlines Plus Jakarta Sans, body Inter. Pill navigation, pill buttons, card radius 18px, panel radius 24px, soft shadow. Same header and footer as the existing GRISTORA pages: wordmark left; nav Product, Breweries, Homebrewers, Pricing, How it works, Technical principles, FAQ; footer "© 2026 GRISTORA · Made for the craft of brewing."
- Copy: use ONLY the copy given in the prompt, word for word. You may shorten nothing and add nothing except navigation labels and the short UI labels listed.
- FORBIDDEN (never generate): testimonials, quotes from customers, customer logos, star ratings, user counts, metrics or percentages, prices or "free" offers, "start trial", "sign up", "book a demo", "live", "available now", "production-ready", "AI", "predictive", "machine learning", forecasts, telemetry dashboards with live numbers, IoT/SCADA/PLC claims, named hardware brands (Tilt, iSpindel, Plaato, etc.), certifications (SOC 2, ISO, GDPR-compliant), uptime/SLA, fake product screenshots with realistic data.
- Illustrations: abstract editorial shapes only (grain, vessel silhouettes, lines). If you show a UI-like panel, label it "Concept" and use placeholder bars, not numbers.
- Accessibility: WCAG 2.1 AA contrast, visible focus, 44px touch targets, one H1 per page, no text in images.
- Output for each screen: a desktop version (1280px wide) AND a mobile version (390px wide). Mobile: hamburger menu, single column, no horizontal scroll, sticky header allowed.

TASK: Generate the following 2 feature detail pages. All 16 feature pages share ONE template; keep it identical to earlier feature pages in this project (if this is the first, define it cleanly so later pages reuse it).

SCREEN: Feature detail page "/features/planning-scheduling/"
Layout: breadcrumb (Home › Product › Planning & scheduling); hero with eyebrow, H1, summary; two pill tags rows ("For": audiences, "Roadmap stage": stages) styled as neutral chips (not as badges of availability); then one card per section in a 2-column grid on desktop (1 column mobile), each card with heading, intro line and a checklist; then a dark forest band with "Explore all features" (link to /features/) and "How it works" (link to /how-it-works/); then footer with "All features ↑".
COPY (use exactly):
Eyebrow: PLAN THE WORK
H1: Planning & scheduling
Summary: Plan backwards from a target packaging date, understand the time required for each step and see how batches fit around equipment availability.
For: Breweries, Homebrewers
Roadmap stage: Homebrew MVP, Business core, Business second release

Section — Calendar and backward planning
Intro: Work from the date you want the beer ready.
- Plan brew day, fermentation steps, dry hop, cooling and packaging dates
- Schedule reminders from fermentation profiles
- Estimate heating and cooling time from power, volume and cooling equipment
- Use past equipment performance to improve timing estimates
- Business tank calendar with capacity and occupancy conflict warnings

Section — Parallel batches
Intro: Keep concurrent work visible.
- See batches by stage and upcoming task
- Single “Today” list of tasks across all active batches
- Weekly tank calendar and daily tank-round printout
- Identify tank conflicts before assigning a batch
- Business tank-emptying suggestions and capacity optimisation

Section — Flexible views
Intro: The business production board adapts to the team.
- Editable dashboard cards and ordering
- Role-specific default views
- Saved views such as active fermentations only
- Operator-focused Today view and wall-mounted display mode


=====

SCREEN: Feature detail page "/features/inventory-costing/"
Layout: breadcrumb (Home › Product › Inventory, ingredients & costing); hero with eyebrow, H1, summary; two pill tags rows ("For": audiences, "Roadmap stage": stages) styled as neutral chips (not as badges of availability); then one card per section in a 2-column grid on desktop (1 column mobile), each card with heading, intro line and a checklist; then a dark forest band with "Explore all features" (link to /features/) and "How it works" (link to /how-it-works/); then footer with "All features ↑".
COPY (use exactly):
Eyebrow: MATERIALS AND COST
H1: Inventory, ingredients & costing
Summary: Manage ingredients from a simple home stock list to professional lot-level traceability, with recipe and batch costs grounded in actual material values.
For: Breweries, Homebrewers
Roadmap stage: Homebrew MVP, Business core, Business second release, Under consideration

Section — Ingredient library
Intro: A shared ingredient catalogue supports recipe design and purchasing.
- Malt, fermentables, hops, yeast, water and miscellaneous ingredients
- Detailed ingredient properties and custom ingredients
- Ingredient substitutions and stock-aware recipe planning
- Ingredient-specific units and regional display
- Custom malt analysis and certificate of analysis (COA) in business mode

Section — Stock and receiving
Intro: Stock handling scales with the workspace mode.
- Home stock quantities, minimum levels and shopping list
- Business stock lots, supplier details, reservations and FIFO valuation
- Goods receiving, barcode support and stock counts
- Movement ledger for receipts, usage, transfers and adjustments
- Lot availability for production

Section — Recipe and batch costs
Intro: Understand material cost at the level that matters.
- Optional recipe cost for homebrewers
- Business batch and per-litre cost using FIFO lot prices
- Bottle and serving cost based on package size
- Cost estimates based on FIFO lot prices and recorded quantities

```

---

## Batch 6 — production-management, quality-laboratory

```text
GLOBAL RULES (apply to every screen in this batch):
- Brand: GRISTORA, descriptor "Brewing Intelligence". Brewing software for professional breweries, with a separate Homebrewers path. English (US/UK audience).
- Visual system (already approved, keep it): page #F7F6F3, deep forest #0B2F26 (text and dark bands), white cards #FFFFFF, muted text #475569, soft line rgba(11,47,38,.18), amber fill #F4A261 for pill CTAs and accents (text on amber is #0B2F26), amber text tone #8B4A1F on light backgrounds, olive #7A8B75 for minor accents, band #F0EFEB. Headlines Plus Jakarta Sans, body Inter. Pill navigation, pill buttons, card radius 18px, panel radius 24px, soft shadow. Same header and footer as the existing GRISTORA pages: wordmark left; nav Product, Breweries, Homebrewers, Pricing, How it works, Technical principles, FAQ; footer "© 2026 GRISTORA · Made for the craft of brewing."
- Copy: use ONLY the copy given in the prompt, word for word. You may shorten nothing and add nothing except navigation labels and the short UI labels listed.
- FORBIDDEN (never generate): testimonials, quotes from customers, customer logos, star ratings, user counts, metrics or percentages, prices or "free" offers, "start trial", "sign up", "book a demo", "live", "available now", "production-ready", "AI", "predictive", "machine learning", forecasts, telemetry dashboards with live numbers, IoT/SCADA/PLC claims, named hardware brands (Tilt, iSpindel, Plaato, etc.), certifications (SOC 2, ISO, GDPR-compliant), uptime/SLA, fake product screenshots with realistic data.
- Illustrations: abstract editorial shapes only (grain, vessel silhouettes, lines). If you show a UI-like panel, label it "Concept" and use placeholder bars, not numbers.
- Accessibility: WCAG 2.1 AA contrast, visible focus, 44px touch targets, one H1 per page, no text in images.
- Output for each screen: a desktop version (1280px wide) AND a mobile version (390px wide). Mobile: hamburger menu, single column, no horizontal scroll, sticky header allowed.

TASK: Generate the following 2 feature detail pages. All 16 feature pages share ONE template; keep it identical to earlier feature pages in this project (if this is the first, define it cleanly so later pages reuse it).

SCREEN: Feature detail page "/features/production-management/"
Layout: breadcrumb (Home › Product › Brewery production management); hero with eyebrow, H1, summary; two pill tags rows ("For": audiences, "Roadmap stage": stages) styled as neutral chips (not as badges of availability); then one card per section in a 2-column grid on desktop (1 column mobile), each card with heading, intro line and a checklist; then a note band; then a dark forest band with "Explore all features" (link to /features/) and "How it works" (link to /how-it-works/); then footer with "All features ↑".
COPY (use exactly):
Eyebrow: BUSINESS MODE
H1: Brewery production management
Summary: A production workspace for small breweries, focused on recipes, batches, tanks, people and records—not a general-purpose ERP.
For: Breweries
Roadmap stage: Business core, Business second release

Section — Production board and operator view
Intro: Give each role a useful view of what needs attention today.
- Production board organised by batch stage and tank
- Today task list for operators
- Role-based dashboard and saved views
- Operator display for production-floor iPads
- Daily tank rounds and printable weekly tank calendar

Section — Tank and batch movements
Intro: Model the physical journey of beer through the brewery.
- Tank detail with occupancy, status, pressure limit and cleaning status
- Transfer between vessels with volume and loss records
- Split one batch across multiple tanks
- Merge batches with calculated blend properties
- Capacity conflict checks and tank scheduling

Section — Production records
Intro: Keep the planned process and actual work connected.
- Operator identity, step completion and actual measurements
- Deviation records and corrective decisions
- Batch traceability from ingredients through packaging, with lot-to-batch recall reports
- Multiple operators and simultaneous active batches
- Audit trail for changes to closed production records
- Warnings when a tank without a valid cleaning record is assigned
- Shift handover of open alerts and production notes

Note: GRISTORA is not an ERP. Sales, orders, distribution, keg tracking, CRM and accounting are outside the product scope.

=====

SCREEN: Feature detail page "/features/quality-laboratory/"
Layout: breadcrumb (Home › Product › Quality, laboratory & release); hero with eyebrow, H1, summary; two pill tags rows ("For": audiences, "Roadmap stage": stages) styled as neutral chips (not as badges of availability); then one card per section in a 2-column grid on desktop (1 column mobile), each card with heading, intro line and a checklist; then a dark forest band with "Explore all features" (link to /features/) and "How it works" (link to /how-it-works/); then footer with "All features ↑".
COPY (use exactly):
Eyebrow: QUALITY MANAGEMENT
H1: Quality, laboratory & release
Summary: Professional quality workflows are built around product specifications, lab results, tasting and a recorded release decision before packaging.
For: Breweries
Roadmap stage: Business core, Business second release, Under consideration

Section — Product specifications
Intro: Define measurable target ranges for each beer.
- Recipe-linked specifications for ABV, OG, FG, IBU, colour and pH
- Dissolved oxygen, turbidity, diacetyl and microbiology parameters
- Out-of-range warnings and deviation records
- Quality review linked to the batch and its recipe version

Section — Laboratory and sensory records
Intro: Keep analytical and tasting results with the batch.
- Laboratory results for pH, dissolved oxygen, CO₂, ABV and microbiology
- Tasting notes, sensory score and descriptive attributes
- Correction records rather than silent edits to historical results
- Sensory panel and triangle-test workflows

Section — Release and process control
Intro: Make the decision to package explicit and traceable.
- Quality hold, rejection and release decisions with reasons
- Release approval before packaging in business workflows
- HACCP checkpoints and dual sign-off

```

---

## Batch 7 — packaging-compliance, team-notifications

```text
GLOBAL RULES (apply to every screen in this batch):
- Brand: GRISTORA, descriptor "Brewing Intelligence". Brewing software for professional breweries, with a separate Homebrewers path. English (US/UK audience).
- Visual system (already approved, keep it): page #F7F6F3, deep forest #0B2F26 (text and dark bands), white cards #FFFFFF, muted text #475569, soft line rgba(11,47,38,.18), amber fill #F4A261 for pill CTAs and accents (text on amber is #0B2F26), amber text tone #8B4A1F on light backgrounds, olive #7A8B75 for minor accents, band #F0EFEB. Headlines Plus Jakarta Sans, body Inter. Pill navigation, pill buttons, card radius 18px, panel radius 24px, soft shadow. Same header and footer as the existing GRISTORA pages: wordmark left; nav Product, Breweries, Homebrewers, Pricing, How it works, Technical principles, FAQ; footer "© 2026 GRISTORA · Made for the craft of brewing."
- Copy: use ONLY the copy given in the prompt, word for word. You may shorten nothing and add nothing except navigation labels and the short UI labels listed.
- FORBIDDEN (never generate): testimonials, quotes from customers, customer logos, star ratings, user counts, metrics or percentages, prices or "free" offers, "start trial", "sign up", "book a demo", "live", "available now", "production-ready", "AI", "predictive", "machine learning", forecasts, telemetry dashboards with live numbers, IoT/SCADA/PLC claims, named hardware brands (Tilt, iSpindel, Plaato, etc.), certifications (SOC 2, ISO, GDPR-compliant), uptime/SLA, fake product screenshots with realistic data.
- Illustrations: abstract editorial shapes only (grain, vessel silhouettes, lines). If you show a UI-like panel, label it "Concept" and use placeholder bars, not numbers.
- Accessibility: WCAG 2.1 AA contrast, visible focus, 44px touch targets, one H1 per page, no text in images.
- Output for each screen: a desktop version (1280px wide) AND a mobile version (390px wide). Mobile: hamburger menu, single column, no horizontal scroll, sticky header allowed.

TASK: Generate the following 2 feature detail pages. All 16 feature pages share ONE template; keep it identical to earlier feature pages in this project (if this is the first, define it cleanly so later pages reuse it).

SCREEN: Feature detail page "/features/packaging-compliance/"
Layout: breadcrumb (Home › Product › Packaging, CIP & compliance); hero with eyebrow, H1, summary; two pill tags rows ("For": audiences, "Roadmap stage": stages) styled as neutral chips (not as badges of availability); then one card per section in a 2-column grid on desktop (1 column mobile), each card with heading, intro line and a checklist; then a note band; then a dark forest band with "Explore all features" (link to /features/) and "How it works" (link to /how-it-works/); then footer with "All features ↑".
COPY (use exactly):
Eyebrow: PACKAGING AND RECORDS
H1: Packaging, CIP & compliance
Summary: Packaging records, vessel cleaning and jurisdiction-aware reporting support professional production, with clear boundaries around what the system does not do.
For: Breweries
Roadmap stage: Business core, Business second release

Section — Packaging runs and containers
Intro: Record what was packaged, in which format and how much.
- Bottle, can, keg and UK cask container types with configurable sizes
- Packaging quantities, litres, losses, packaging efficiency and lot codes
- Multiple container types within one batch
- Bottle priming, keg pressure carbonation, canning from BBT and cask conditioning guidance
- Package material and empty weight for packaging reporting

Section — Cleaning and cellar process
Intro: Connect cleaning records to vessel readiness.
- CIP chemical, concentration, temperature, duration and verification
- Tank cleaning status and last-cleaned record
- Warnings when a tank without a valid cleaning record is assigned
- Spunding and forced carbonation planning
- Cask conditioning and finings calculations for the UK market

Section — Labels and regulatory reports
Intro: Support production-side reporting without pretending to replace accounting or sales systems.
- Country-specific tax calculation methods and reporting templates
- US TTB, UK HMRC, Australia ATO, Germany Zoll and Türkiye reporting scope
- ABV or Plato-based tax calculations according to jurisdiction
- Label information such as ABV, lot code, allergens and nutrition where requirements are confirmed
- Deposit-return and packaging/EPR reports

Note: Sales and shipment quantities may need manual entry or file import because sales and distribution are outside scope.

=====

SCREEN: Feature detail page "/features/team-notifications/"
Layout: breadcrumb (Home › Product › Teams, roles & notifications); hero with eyebrow, H1, summary; two pill tags rows ("For": audiences, "Roadmap stage": stages) styled as neutral chips (not as badges of availability); then one card per section in a 2-column grid on desktop (1 column mobile), each card with heading, intro line and a checklist; then a dark forest band with "Explore all features" (link to /features/) and "How it works" (link to /how-it-works/); then footer with "All features ↑".
COPY (use exactly):
Eyebrow: PEOPLE AND WORKFLOWS
H1: Teams, roles & notifications
Summary: One workspace supports a solo brewer and a multi-role production team, with notifications routed to the right person and actions recorded.
For: Breweries, Homebrewers
Roadmap stage: Homebrew MVP, Business core

Section — Home and business workspaces
Intro: The workspace chooses Home or Business mode without splitting the underlying recipe and batch history.
- Select a mode during onboarding and change it later in workspace settings
- Home mode keeps advanced business fields out of the way
- Optional invitations to collaborate on a recipe or batch in Home mode
- Switching modes preserves recipes and history; hidden business records are not deleted

Section — Team, roles and shared devices
Intro: Business workspaces support role-based production work.
- Invite users with time-limited invitations and assign one or more roles
- Owner, brewmaster, operator, quality, warehouse and read-only auditor roles
- Custom roles, copied roles, permission groups and optional vessel/area scope
- Person-by-permission overview and role-specific opening screens
- Shared production iPad with personal PIN, fast user switching and automatic lock

Section — Notifications and approvals
Intro: A single notification record stays consistent across devices.
- In-app inbox, push notifications and optional daily email summary
- Three priority levels, quiet hours, grouped alerts and notification history
- Acknowledge, snooze, ignore or resolve notifications with an event record
- Business shift routing, escalation and shift handover
- Recipe approval, quality release and other role-based approval queues

```

---

## Batch 8 — connected-devices, imports-exports-tools

```text
GLOBAL RULES (apply to every screen in this batch):
- Brand: GRISTORA, descriptor "Brewing Intelligence". Brewing software for professional breweries, with a separate Homebrewers path. English (US/UK audience).
- Visual system (already approved, keep it): page #F7F6F3, deep forest #0B2F26 (text and dark bands), white cards #FFFFFF, muted text #475569, soft line rgba(11,47,38,.18), amber fill #F4A261 for pill CTAs and accents (text on amber is #0B2F26), amber text tone #8B4A1F on light backgrounds, olive #7A8B75 for minor accents, band #F0EFEB. Headlines Plus Jakarta Sans, body Inter. Pill navigation, pill buttons, card radius 18px, panel radius 24px, soft shadow. Same header and footer as the existing GRISTORA pages: wordmark left; nav Product, Breweries, Homebrewers, Pricing, How it works, Technical principles, FAQ; footer "© 2026 GRISTORA · Made for the craft of brewing."
- Copy: use ONLY the copy given in the prompt, word for word. You may shorten nothing and add nothing except navigation labels and the short UI labels listed.
- FORBIDDEN (never generate): testimonials, quotes from customers, customer logos, star ratings, user counts, metrics or percentages, prices or "free" offers, "start trial", "sign up", "book a demo", "live", "available now", "production-ready", "AI", "predictive", "machine learning", forecasts, telemetry dashboards with live numbers, IoT/SCADA/PLC claims, named hardware brands (Tilt, iSpindel, Plaato, etc.), certifications (SOC 2, ISO, GDPR-compliant), uptime/SLA, fake product screenshots with realistic data.
- Illustrations: abstract editorial shapes only (grain, vessel silhouettes, lines). If you show a UI-like panel, label it "Concept" and use placeholder bars, not numbers.
- Accessibility: WCAG 2.1 AA contrast, visible focus, 44px touch targets, one H1 per page, no text in images.
- Output for each screen: a desktop version (1280px wide) AND a mobile version (390px wide). Mobile: hamburger menu, single column, no horizontal scroll, sticky header allowed.

TASK: Generate the following 2 feature detail pages. All 16 feature pages share ONE template; keep it identical to earlier feature pages in this project (if this is the first, define it cleanly so later pages reuse it).

SCREEN: Feature detail page "/features/connected-devices/"
Layout: breadcrumb (Home › Product › Connected devices & integrations); hero with eyebrow, H1, summary; two pill tags rows ("For": audiences, "Roadmap stage": stages) styled as neutral chips (not as badges of availability); then one card per section in a 2-column grid on desktop (1 column mobile), each card with heading, intro line and a checklist; then a dark forest band with "Explore all features" (link to /features/) and "How it works" (link to /how-it-works/); then footer with "All features ↑".
COPY (use exactly):
Eyebrow: CONNECTED BREWING
H1: Connected devices & integrations
Summary: Read-only fermentation device data sits alongside professional sensor and industrial gateway support.
For: Breweries, Homebrewers
Roadmap stage: Homebrew MVP, Business second release, Later, Under consideration

Section — Homebrewing devices
Intro: Connect common fermentation monitors through dedicated adapters.
- Tilt hydrometers, iSpindel and RAPT devices
- Webhook data endpoint for TiltBridge, Pitch and compatible bridges
- Device-specific keys that can be renewed
- Device-to-batch pairing and calibration records
- Show connection and reading status

Section — Professional sensors
Intro: Business mode adds tank-level monitoring.
- Gravity, pH, dissolved oxygen and pressure readings
- Tank assignment and basic connection status
- API or CSV ingestion for professional sensors
- Industrial gateway read support for Modbus TCP, OPC UA and MQTT
- Sending control commands to equipment is outside scope

Section — Data use and reliability
Intro: Device data should remain useful when a sensor connection is interrupted.
- Buffer sensor readings at the device or gateway and forward when connected
- Show source labels and calibration context alongside measurements
- Keep device adapters independent of any single manufacturer
- Use readings for fermentation trends and configured threshold alerts
- Open API and professional production-data webhooks for Business mode; Homebrew device webhooks are separate
- Industrial gateway support is read-only; equipment command and control is outside scope


=====

SCREEN: Feature detail page "/features/imports-exports-tools/"
Layout: breadcrumb (Home › Product › Import, export & brewing tools); hero with eyebrow, H1, summary; two pill tags rows ("For": audiences, "Roadmap stage": stages) styled as neutral chips (not as badges of availability); then one card per section in a 2-column grid on desktop (1 column mobile), each card with heading, intro line and a checklist; then a dark forest band with "Explore all features" (link to /features/) and "How it works" (link to /how-it-works/); then footer with "All features ↑".
COPY (use exactly):
Eyebrow: PORTABILITY AND UTILITIES
H1: Import, export & brewing tools
Summary: Bring existing recipe work into GRISTORA, move your data out when needed and use focused brewing tools alongside recipe and batch workflows.
For: Breweries, Homebrewers
Roadmap stage: Homebrew MVP, Homebrew second release

Section — Recipe portability
Intro: Import paths cover established brewing formats.
- BeerXML and BeerJSON import/export
- BeerSmith .bsmx import
- Brewfather and Brewer's Friend recipe exports
- Ingredient, equipment, style and unit matching during import
- Show source-versus-GRISTORA calculation differences before saving

Section — Focused brewing tools
Intro: Focused utilities sit within the relevant recipe or batch context.
- Refractometer correction and gravity tools
- Carbonation and priming calculators for bottles, kegs and casks
- Dilution, volume and unit conversion
- Extract brewing tools for partial boils, top-up water and LME/DME/honey conversions
- Decoction and parti-gyle tools; parti-gyle is available in Home mode
- No-chill IBU adjustment and bottle/keg carbonation tracking in Home mode
- Optional recipe transfer to supported brewing controllers in Home mode

Section — Reports and custom records
Intro: Make useful records portable and printable.
- Brew sheets, tank rounds, fermentation and packaging forms
- Custom brew-sheet fields and layouts in Home mode
- Recipe, batch, equipment, water profile, ingredient, device and support data export
- CSV and JSON exports plus scanned documents and photos in downloadable archives
- Competition-form output and custom recipe/batch fields in Home mode
- Bring in laboratory water reports in Home mode

```

---

## Batch 9 — platform-security-data, analytics-reports

```text
GLOBAL RULES (apply to every screen in this batch):
- Brand: GRISTORA, descriptor "Brewing Intelligence". Brewing software for professional breweries, with a separate Homebrewers path. English (US/UK audience).
- Visual system (already approved, keep it): page #F7F6F3, deep forest #0B2F26 (text and dark bands), white cards #FFFFFF, muted text #475569, soft line rgba(11,47,38,.18), amber fill #F4A261 for pill CTAs and accents (text on amber is #0B2F26), amber text tone #8B4A1F on light backgrounds, olive #7A8B75 for minor accents, band #F0EFEB. Headlines Plus Jakarta Sans, body Inter. Pill navigation, pill buttons, card radius 18px, panel radius 24px, soft shadow. Same header and footer as the existing GRISTORA pages: wordmark left; nav Product, Breweries, Homebrewers, Pricing, How it works, Technical principles, FAQ; footer "© 2026 GRISTORA · Made for the craft of brewing."
- Copy: use ONLY the copy given in the prompt, word for word. You may shorten nothing and add nothing except navigation labels and the short UI labels listed.
- FORBIDDEN (never generate): testimonials, quotes from customers, customer logos, star ratings, user counts, metrics or percentages, prices or "free" offers, "start trial", "sign up", "book a demo", "live", "available now", "production-ready", "AI", "predictive", "machine learning", forecasts, telemetry dashboards with live numbers, IoT/SCADA/PLC claims, named hardware brands (Tilt, iSpindel, Plaato, etc.), certifications (SOC 2, ISO, GDPR-compliant), uptime/SLA, fake product screenshots with realistic data.
- Illustrations: abstract editorial shapes only (grain, vessel silhouettes, lines). If you show a UI-like panel, label it "Concept" and use placeholder bars, not numbers.
- Accessibility: WCAG 2.1 AA contrast, visible focus, 44px touch targets, one H1 per page, no text in images.
- Output for each screen: a desktop version (1280px wide) AND a mobile version (390px wide). Mobile: hamburger menu, single column, no horizontal scroll, sticky header allowed.

TASK: Generate the following 2 feature detail pages. All 16 feature pages share ONE template; keep it identical to earlier feature pages in this project (if this is the first, define it cleanly so later pages reuse it).

SCREEN: Feature detail page "/features/platform-security-data/"
Layout: breadcrumb (Home › Product › Platform, security & data ownership); hero with eyebrow, H1, summary; two pill tags rows ("For": audiences, "Roadmap stage": stages) styled as neutral chips (not as badges of availability); then one card per section in a 2-column grid on desktop (1 column mobile), each card with heading, intro line and a checklist; then a dark forest band with "Explore all features" (link to /features/) and "How it works" (link to /how-it-works/); then footer with "All features ↑".
COPY (use exactly):
Eyebrow: PLATFORM
H1: Platform, security & data ownership
Summary: The product is one connected system across Apple devices and desktop web, with a shared server-side engine and explicit data ownership controls.
For: Breweries, Homebrewers
Roadmap stage: Homebrew MVP, Business core

Section — Where GRISTORA runs
Intro: Each client has its own role.
- iPad: full-featured native SwiftUI app
- iPhone: brew day, fermentation, stock and shopping workflows, plus recipe viewing and simple editing
- Web: full-featured React and TypeScript experience for Mac and PC browsers
- Apple Watch is not supported
- Android and separate Mac/Windows desktop apps are not in scope

Section — One engine and one source of truth
Intro: The same calculation and business rules serve every client.
- Rust calculation engine runs server-side; calculation code is not shipped to clients
- Shared API and data model across web and Apple applications
- Online-first operation with a limited downloaded brew-day package for offline timers and measurement entry
- No general offline synchronisation; queued brew-day measurements upload when connectivity returns
- Versioned calculation results and immutable recipe/batch references

Section — Security and data ownership
Intro: Security and portability are part of the platform.
- Sign in with Apple or Google, with one-time email links as a fallback; no password-based signup
- Passkeys for business administrators, session management and remote sign-out
- Workspace separation, server-side permissions and append-only audit records
- Encrypted storage, protected file access and time-limited support access with customer consent
- Export your recipes, batches, measurements, stock, equipment and files; request account deletion

Section — Support and service
Intro: Customer support remains part of the product experience.
- Searchable help and support request inbox
- Customer-visible request history and replies
- Optional, time-limited support access with a reason and audit history
- Internal support workspace separated from the customer application
- System status and support diagnostics

Section — Workspace setup, preferences and subscription
Intro: The first-run and account experience centres on a workspace rather than a separate registration form.
- Guided first launch: choose Home or Business mode, data region and equipment profile
- Global search across recipes, batches, ingredients and stock
- Regional presets, category-specific units, local number/date formats and English/Turkish language support
- Theme, appearance, notifications, equipment, calculation and style-guide settings
- Subscription and entitlement management
- Account linking, session management, remote sign-out, account export and deletion

Section — Apple and web experience
Intro: System-level conveniences where the platform supports them.
- iOS Live Activities, Dynamic Island, widgets, time-sensitive notifications and document scanning
- Siri and Shortcuts integration as part of the Apple experience
- Apple Watch is not supported; Android and separate desktop apps are out of scope
- Face ID or PIN app lock, shared-device PIN and administrator passkeys
- Encrypted backups, data portability and customer-visible support access history


=====

SCREEN: Feature detail page "/features/analytics-reports/"
Layout: breadcrumb (Home › Product › Reports, insights & sustainability); hero with eyebrow, H1, summary; two pill tags rows ("For": audiences, "Roadmap stage": stages) styled as neutral chips (not as badges of availability); then one card per section in a 2-column grid on desktop (1 column mobile), each card with heading, intro line and a checklist; then a dark forest band with "Explore all features" (link to /features/) and "How it works" (link to /how-it-works/); then footer with "All features ↑".
COPY (use exactly):
Eyebrow: REPORTING AND INSIGHTS
H1: Reports, insights & sustainability
Summary: Operational reporting covers batches, recipes and brewery production.
For: Breweries, Homebrewers
Roadmap stage: Homebrew MVP, Business core, Business second release, Under consideration

Section — Batch and recipe learning
Intro: Use actual results to understand repeatability.
- Plan-versus-actual batch summary and tasting notes
- Compare batches made from the same recipe
- Trends for gravity, efficiency and tasting scores in Home mode

Section — Brewery reporting
Intro: Business reporting focuses on production and material performance.
- Brewhouse, conversion and lautering efficiency
- Volume loss from brew day through packaging
- FIFO ingredient cost and cost per litre
- Tank utilisation, batch throughput and production dashboard metrics
- Lot traceability and recall report from ingredient lot to packaging run

```

---

## Batch 10 — feature directory and 404

```text
GLOBAL RULES (apply to every screen in this batch):
- Brand: GRISTORA, descriptor "Brewing Intelligence". Brewing software for professional breweries, with a separate Homebrewers path. English (US/UK audience).
- Visual system (already approved, keep it): page #F7F6F3, deep forest #0B2F26 (text and dark bands), white cards #FFFFFF, muted text #475569, soft line rgba(11,47,38,.18), amber fill #F4A261 for pill CTAs and accents (text on amber is #0B2F26), amber text tone #8B4A1F on light backgrounds, olive #7A8B75 for minor accents, band #F0EFEB. Headlines Plus Jakarta Sans, body Inter. Pill navigation, pill buttons, card radius 18px, panel radius 24px, soft shadow. Same header and footer as the existing GRISTORA pages: wordmark left; nav Product, Breweries, Homebrewers, Pricing, How it works, Technical principles, FAQ; footer "© 2026 GRISTORA · Made for the craft of brewing."
- Copy: use ONLY the copy given in the prompt, word for word. You may shorten nothing and add nothing except navigation labels and the short UI labels listed.
- FORBIDDEN (never generate): testimonials, quotes from customers, customer logos, star ratings, user counts, metrics or percentages, prices or "free" offers, "start trial", "sign up", "book a demo", "live", "available now", "production-ready", "AI", "predictive", "machine learning", forecasts, telemetry dashboards with live numbers, IoT/SCADA/PLC claims, named hardware brands (Tilt, iSpindel, Plaato, etc.), certifications (SOC 2, ISO, GDPR-compliant), uptime/SLA, fake product screenshots with realistic data.
- Illustrations: abstract editorial shapes only (grain, vessel silhouettes, lines). If you show a UI-like panel, label it "Concept" and use placeholder bars, not numbers.
- Accessibility: WCAG 2.1 AA contrast, visible focus, 44px touch targets, one H1 per page, no text in images.
- Output for each screen: a desktop version (1280px wide) AND a mobile version (390px wide). Mobile: hamburger menu, single column, no horizontal scroll, sticky header allowed.

SCREEN 1: Feature directory "/features/" — a grid of 16 cards (3 columns desktop, 1 column mobile). Each card: eyebrow, title, the first sentence of the summary, audience chips, link "Read more". Add a filter row with chips "All", "Breweries", "Homebrewers" (visual only). Cards in this order:
- RECIPES · Recipe design & management · From a guided first recipe to a deeply configurable professional formulation, GRISTORA keeps recipes understandable from version to version. · For: Breweries, Homebrewers · /features/recipe-design/
- THE CALCULATION ENGINE · Brewing calculations · A shared, server-side calculation engine serves the web and Apple apps, with clear inputs, consistent units and an explanation behind every result. · For: Breweries, Homebrewers · /features/brewing-calculations/
- PROCESS DESIGN · Water, mash & yeast · Plan water chemistry, mash steps and yeast preparation as connected parts of the recipe rather than isolated calculators. · For: Breweries, Homebrewers · /features/water-mash-yeast/
- BREW DAY · Brew day & production records · A brew-day workspace keeps the recipe plan separate from what actually happened, while remaining usable beside wet equipment and paper records. · For: Breweries, Homebrewers · /features/brew-day/
- FERMENTATION · Fermentation & batch tracking · Follow each batch from brew day through fermentation, conditioning and packaging, using device readings or manual measurements. · For: Breweries, Homebrewers · /features/fermentation-monitoring/
- EQUIPMENT · Equipment & calibration · Equipment profiles reflect the real behaviour of a home setup or professional brewhouse, and improve through measured batch outcomes. · For: Breweries, Homebrewers · /features/equipment-calibration/
- PLAN THE WORK · Planning & scheduling · Plan backwards from a target packaging date, understand the time required for each step and see how batches fit around equipment availability. · For: Breweries, Homebrewers · /features/planning-scheduling/
- MATERIALS AND COST · Inventory, ingredients & costing · Manage ingredients from a simple home stock list to professional lot-level traceability, with recipe and batch costs grounded in actual material values. · For: Breweries, Homebrewers · /features/inventory-costing/
- BUSINESS MODE · Brewery production management · A production workspace for small breweries, focused on recipes, batches, tanks, people and records—not a general-purpose ERP. · For: Breweries · /features/production-management/
- QUALITY MANAGEMENT · Quality, laboratory & release · Professional quality workflows are built around product specifications, lab results, tasting and a recorded release decision before packaging. · For: Breweries · /features/quality-laboratory/
- PACKAGING AND RECORDS · Packaging, CIP & compliance · Packaging records, vessel cleaning and jurisdiction-aware reporting support professional production, with clear boundaries around what the system does not do. · For: Breweries · /features/packaging-compliance/
- PEOPLE AND WORKFLOWS · Teams, roles & notifications · One workspace supports a solo brewer and a multi-role production team, with notifications routed to the right person and actions recorded. · For: Breweries, Homebrewers · /features/team-notifications/
- CONNECTED BREWING · Connected devices & integrations · Read-only fermentation device data sits alongside professional sensor and industrial gateway support. · For: Breweries, Homebrewers · /features/connected-devices/
- PORTABILITY AND UTILITIES · Import, export & brewing tools · Bring existing recipe work into GRISTORA, move your data out when needed and use focused brewing tools alongside recipe and batch workflows. · For: Breweries, Homebrewers · /features/imports-exports-tools/
- PLATFORM · Platform, security & data ownership · The product is one connected system across Apple devices and desktop web, with a shared server-side engine and explicit data ownership controls. · For: Breweries, Homebrewers · /features/platform-security-data/
- REPORTING AND INSIGHTS · Reports, insights & sustainability · Operational reporting covers batches, recipes and brewery production. · For: Breweries, Homebrewers · /features/analytics-reports/

SCREEN 2: 404 page. COPY (use exactly):
Eyebrow: 404 · PAGE NOT FOUND
H1: This page seems to have gone off the boil.  (style "gone off the boil." in italic amber-text tone)
Body: The address may be out of date, or the page may have moved. Head back to GRISTORA and choose the brewing path that fits you.
Primary pill button: Back to GRISTORA ↗
Text links: Professional breweries · Homebrewers
Add one calm abstract illustration (an empty kettle silhouette with a thin steam line), no text in it. Minimal footer.
```

---

## Batch 11 — consistency pass

```text
GLOBAL RULES (apply to every screen in this batch):
- Brand: GRISTORA, descriptor "Brewing Intelligence". Brewing software for professional breweries, with a separate Homebrewers path. English (US/UK audience).
- Visual system (already approved, keep it): page #F7F6F3, deep forest #0B2F26 (text and dark bands), white cards #FFFFFF, muted text #475569, soft line rgba(11,47,38,.18), amber fill #F4A261 for pill CTAs and accents (text on amber is #0B2F26), amber text tone #8B4A1F on light backgrounds, olive #7A8B75 for minor accents, band #F0EFEB. Headlines Plus Jakarta Sans, body Inter. Pill navigation, pill buttons, card radius 18px, panel radius 24px, soft shadow. Same header and footer as the existing GRISTORA pages: wordmark left; nav Product, Breweries, Homebrewers, Pricing, How it works, Technical principles, FAQ; footer "© 2026 GRISTORA · Made for the craft of brewing."
- Copy: use ONLY the copy given in the prompt, word for word. You may shorten nothing and add nothing except navigation labels and the short UI labels listed.
- FORBIDDEN (never generate): testimonials, quotes from customers, customer logos, star ratings, user counts, metrics or percentages, prices or "free" offers, "start trial", "sign up", "book a demo", "live", "available now", "production-ready", "AI", "predictive", "machine learning", forecasts, telemetry dashboards with live numbers, IoT/SCADA/PLC claims, named hardware brands (Tilt, iSpindel, Plaato, etc.), certifications (SOC 2, ISO, GDPR-compliant), uptime/SLA, fake product screenshots with realistic data.
- Illustrations: abstract editorial shapes only (grain, vessel silhouettes, lines). If you show a UI-like panel, label it "Concept" and use placeholder bars, not numbers.
- Accessibility: WCAG 2.1 AA contrast, visible focus, 44px touch targets, one H1 per page, no text in images.
- Output for each screen: a desktop version (1280px wide) AND a mobile version (390px wide). Mobile: hamburger menu, single column, no horizontal scroll, sticky header allowed.

TASK: Review every screen in this project. Fix, without changing copy: header and footer identical on all pages; same card radius, spacing scale (8px grid) and button styles; same feature-page template on all 16 feature pages; every desktop screen has a 390px mobile pair. Remove anything that breaks the FORBIDDEN list (testimonials, numbers, prices, AI/predictive, hardware brands, certifications, live dashboards). Produce a final "Screen index" screen that lists every screen name and its route.
```

