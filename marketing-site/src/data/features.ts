export type FeatureAudience = "Breweries" | "Homebrewers";
export type FeatureModule = "core" | "devices" | "stock" | "production" | "quality" | "cost-reports" | "tax-compliance" | "team" | "connections";
export type FeatureStage =
  | "Homebrew MVP"
  | "Homebrew second release"
  | "Business core"
  | "Business second release"
  | "Later"
  | "Under consideration";

export type FeatureSection = {
  heading: string;
  intro: string;
  items: string[];
};

export type FeaturePage = {
  slug: string;
  title: string;
  eyebrow: string;
  summary: string;
  audiences: FeatureAudience[];
  stages: FeatureStage[];
  modules: FeatureModule[];
  sections: FeatureSection[];
  note?: string;
};

export const features: FeaturePage[] = [
  {
    slug: "recipe-design",
    modules: ["core"],
    title: "Recipe design & management",
    eyebrow: "RECIPES",
    summary: "From a guided first recipe to a deeply configurable professional formulation, GRISTORA keeps recipes understandable from version to version.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business core"],
    sections: [
      { heading: "Start the way that suits you", intro: "Four entry points lead into one consistent recipe workflow.", items: ["Start from a blank recipe with style- and target-led ingredient suggestions", "Use style-based starter templates and adapt them to your equipment and batch size", "Copy an existing recipe or create a deliberate new version", "Import BeerXML, BeerJSON, BeerSmith .bsmx, Brewfather and Brewer's Friend exports with ingredient, equipment, style and unit matching"] },
      { heading: "A recipe editor built for iteration", intro: "The editor is designed to make changes and their consequences visible.", items: ["Edit fermentables, hops, yeast, miscellaneous ingredients, mash, boil and fermentation details", "Adjust grain percentages and target OG, IBU or colour while keeping selected values in balance", "Convert all-grain recipes to extract and back; substitute ingredients and round quantities", "Scale by batch volume, brewhouse efficiency or equipment while recalculating losses", "Review recipe checks, warnings and one-step correction suggestions"] },
      { heading: "Styles, versions and collaboration", intro: "Keep the reference and history behind a recipe clear.", items: ["BJCP 2021, BJCP mead/cider and Brewers Association style guides, with custom guide support", "Compare style ranges and map a recipe between guides", "Compare two recipes or versions side by side", "Keep immutable published versions; business recipes move through draft, review and approval", "Lock a recipe against accidental edits and preserve the version used by each batch"] },
      { heading: "Recipe library and supported brew types", intro: "Keep a growing recipe collection organised and useful.", items: ["Organise recipes and batches with search, filters, folders and labels", "Pin favourites and choose which columns appear in recipe lists", "Support beer, mead, cider and seltzer recipes, including all-grain, extract, partial mash and BIAB approaches", "Keep recipe notes, colour choices, images and custom fields", "Archive and restore recipe history without silently overwriting published versions"] }
    ]
  },
  {
    slug: "brewing-calculations",
    modules: ["core"],
    title: "Brewing calculations",
    eyebrow: "THE CALCULATION ENGINE",
    summary: "A shared, server-side calculation engine serves the web and Apple apps, with clear inputs, consistent units and an explanation behind every result.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business core", "Business second release", "Under consideration"],
    sections: [
      { heading: "Core recipe calculations", intro: "A broad calculation set supports formulation and review.", items: ["Original and final gravity (OG / FG), apparent attenuation and alcohol by volume (ABV)", "Bitterness (IBU), including Tinseth as the default, late additions, whirlpool and hop-stand behaviour", "Colour estimates with selectable calculation approach", "Mash-temperature and yeast-tolerance adjustments to fermentation estimates", "Batch, kettle, trub and volume-chain calculations"] },
      { heading: "Advanced brewing models", intro: "The engine includes specialist methods and configurable assumptions.", items: ["Hop-ageing and alpha-acid adjustments using supplied inputs", "Starter growth and pitching calculations", "Carbonation and priming for bottles and kegs, including keg headspace and pressure limits", "Mead and wine fermentation calculations, with corrected YAN handling", "Extract, partial-boil, dilution and no-chill calculation tools"] },
      { heading: "Explainable results", intro: "Technical transparency is a product principle.", items: ["A “How it was calculated” view for key results", "Calculation engine version stored with each saved result", "Consistent SI-based internal values with regional display and unit conversion", "Reference test vectors and golden-recipe validation for every engine release"] },
      { heading: "Specialist methods", intro: "Additional methods sit apart from the core calculation set.", items: ["No-chill IBU, hop-oil ratios, BU:GU and alternative IBU and colour methods", "Fruit additions, pH temperature correction and total-acidity adjustment for cider and wine", "Sour and mixed fermentation targets and barrel-ageing calculations", "Keg line-balance tools for hose diameter, length and serving pressure", "Method selection and advanced defaults in calculation settings"] }
    ],
  },
  {
    slug: "water-mash-yeast",
    modules: ["core"],
    title: "Water, mash & yeast",
    eyebrow: "PROCESS DESIGN",
    summary: "Plan water chemistry, mash steps and yeast preparation as connected parts of the recipe rather than isolated calculators.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Under consideration"],
    sections: [
      { heading: "Water profiles", intro: "Build a water profile from source water and a target style or recipe.", items: ["Save source-water and target-water profiles", "Enter a laboratory water report and manage ion concentrations", "Calculate salt additions and acid adjustments", "Review ion balance, hardness and regional hardness units", "Compare water profiles and retain the report used for a recipe"] },
      { heading: "Mash planning", intro: "The mash plan connects ingredient choices, temperatures and volumes.", items: ["Mash schedule with rests, temperatures, durations and infusion steps", "Mash pH estimation using the selected model", "Decoction calculator for mash volume and temperature steps", "Mash water, sparge water and volume calculations", "BIAB-specific equipment and absorption handling"] },
      { heading: "Yeast preparation", intro: "Keep yeast requirements and fermentation expectations visible.", items: ["Yeast selection and age details", "Starter planning, including two-stage starters", "Dry yeast pitching guidance by hectolitre in business mode", "Fermentation profiles and target conditions", "Harvest and generation tracking in the business core; advanced home mode can enable yeast management"] }
    ]
  },
  {
    slug: "brew-day",
    modules: ["core"],
    title: "Brew day & production records",
    eyebrow: "BREW DAY",
    summary: "A brew-day workspace keeps the recipe plan separate from what actually happened, while remaining usable beside wet equipment and paper records.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business core"],
    sections: [
      { heading: "A guided brew-day timeline", intro: "Move through a clear sequence on phone, tablet or web.", items: ["Full-screen step-by-step iPhone workflow and a detailed iPad/web step list", "Optional timers, start/finish controls and editable actual times", "Next-step guidance, target values and measurement entry", "Apple Live Activity, Dynamic Island and time-sensitive notifications on supported devices", "Multiple batches can run at the same time"] },
      { heading: "Plan versus actual", intro: "The batch records reality without silently changing the source recipe.", items: ["Record actual quantities, time, temperature and volume", "Add, skip, reorder or correct steps; record ingredient substitutions and additions", "Mark deviations and recalculate actual OG, IBU and ABV when online", "Compare measured values with targets and receive contextual interpretation", "Optionally turn the actual outcome into a new recipe draft"] },
      { heading: "Paper-friendly workflow", intro: "Digital and paper workflows are designed to work together.", items: ["Printable A4 or US Letter brew sheets with planned and actual fields", "QR code connects a printed sheet to its batch", "Enter paper measurements in the same numbered sequence", "Scan brew sheets with Apple document scanning and review extracted values side by side", "Business workspaces can require scanned records and apply retention rules", "Customise brew-sheet sections and fields in Home mode", "Use quick measurement entry from reminders and supported system shortcuts"] }
    ]
  },
  {
    slug: "fermentation-monitoring",
    modules: ["core"],
    title: "Fermentation & batch tracking",
    eyebrow: "FERMENTATION",
    summary: "Follow each batch from brew day through fermentation, conditioning and packaging, using device readings or manual measurements.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business core"],
    sections: [
      { heading: "A complete batch timeline", intro: "Each brew has its own record and stage history.", items: ["Track planned, brewing, fermenting, conditioning, ready-to-package, packaged and closed stages", "Run multiple batches and fermenters concurrently", "Keep measurements, notes, photos and events on the batch timeline", "Compare plan and actual values in the batch summary", "Record tasting notes and use them to inform future recipe work"] },
      { heading: "Measurements and insights", intro: "Bring readings together in one view.", items: ["Gravity, temperature and pH readings from devices or manual entry", "Charts with measurement-source labels for device, manual and paper readings", "Fermentation progress highlights", "Dry-hop, cold-crash and packaging timing suggestions", "Manual-reading reminders when no sensor is connected"] },
      { heading: "Rules and alerts", intro: "Alerts respond to the process and do not create unnecessary noise.", items: ["Rules such as gravity stable for several days or temperature outside a target", "Three priority levels, quiet hours, grouped alerts and daily summaries", "One notification inbox with acknowledge, snooze, ignore and history", "Business shift routing and escalation to the next responsible person", "Quick SG and temperature entry directly from a manual-reading reminder", "Home widgets and tank-focused widgets for at-a-glance fermentation status"] },
      { heading: "Batch comparison and conditioning", intro: "Use completed batches to understand repeatability over time.", items: ["Compare batches made from the same recipe by OG, FG, efficiency and tasting score", "Track bottle and keg conditioning progress towards target carbonation in Home mode", "Turn tasting feedback into a recipe draft in Home mode"] }
    ]
  },
  {
    slug: "equipment-calibration",
    modules: ["core"],
    title: "Equipment & calibration",
    eyebrow: "EQUIPMENT",
    summary: "Equipment profiles reflect the real behaviour of a home setup or professional brewhouse, and improve through measured batch outcomes.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business core"],
    sections: [
      { heading: "Equipment profiles", intro: "Set up the equipment that shapes every calculation.", items: ["Guided equipment setup for a first profile", "Mash tun, kettle, fermenter and packaging vessel details", "Volume losses, grain absorption, evaporation, dead space and cooling shrinkage", "Multiple brewhouses and vessel profiles for business workspaces", "Equipment-specific recipe scaling and efficiency assumptions"] },
      { heading: "Vessel readiness", intro: "Professional vessel details connect equipment planning to production.", items: ["Tank capacity, pressure limit and current occupancy", "Cleaning status and last cleaning record", "Vessel availability in the production calendar", "Equipment calibration records"] }
    ]
  },
  {
    slug: "planning-scheduling",
    modules: ["core","production"],
    title: "Planning & scheduling",
    eyebrow: "PLAN THE WORK",
    summary: "Plan backwards from a target packaging date, understand the time required for each step and see how batches fit around equipment availability.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business core", "Business second release"],
    sections: [
      { heading: "Calendar and backward planning", intro: "Work from the date you want the beer ready.", items: ["Plan brew day, fermentation steps, dry hop, cooling and packaging dates", "Schedule reminders from fermentation profiles", "Estimate heating and cooling time from power, volume and cooling equipment", "Use past equipment performance to improve timing estimates", "Business tank calendar with capacity and occupancy conflict warnings"] },
      { heading: "Parallel batches", intro: "Keep concurrent work visible.", items: ["See batches by stage and upcoming task", "Single “Today” list of tasks across all active batches", "Weekly tank calendar and daily tank-round printout", "Identify tank conflicts before assigning a batch", "Business tank-emptying suggestions and capacity optimisation"] },
      { heading: "Flexible views", intro: "The business production board adapts to the team.", items: ["Editable dashboard cards and ordering", "Role-specific default views", "Saved views such as active fermentations only", "Operator-focused Today view and wall-mounted display mode"] }
    ]
  },
  {
    slug: "inventory-costing",
    modules: ["stock","cost-reports"],
    title: "Inventory, ingredients & costing",
    eyebrow: "MATERIALS AND COST",
    summary: "Manage ingredients from a simple home stock list to professional lot-level traceability, with recipe and batch costs grounded in actual material values.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business core", "Business second release", "Under consideration"],
    sections: [
      { heading: "Ingredient library", intro: "A shared ingredient catalogue supports recipe design and purchasing.", items: ["Malt, fermentables, hops, yeast, water and miscellaneous ingredients", "Detailed ingredient properties and custom ingredients", "Ingredient substitutions and stock-aware recipe planning", "Ingredient-specific units and regional display", "Custom malt analysis and certificate of analysis (COA) in business mode"] },
      { heading: "Stock and receiving", intro: "Stock handling scales with the workspace mode.", items: ["Home stock quantities, minimum levels and shopping list", "Business stock lots, supplier details, reservations and FIFO valuation", "Goods receiving, barcode support and stock counts", "Movement ledger for receipts, usage, transfers and adjustments", "Lot availability for production"] },
      { heading: "Recipe and batch costs", intro: "Understand material cost at the level that matters.", items: ["Optional recipe cost for homebrewers", "Business batch and per-litre cost using FIFO lot prices", "Bottle and serving cost based on package size", "Cost estimates based on FIFO lot prices and recorded quantities"] },
    ]
  },
  {
    slug: "production-management",
    modules: ["production"],
    title: "Brewery production management",
    eyebrow: "BUSINESS MODE",
    summary: "A production workspace for small breweries, focused on recipes, batches, tanks, people and records—not a general-purpose ERP.",
    audiences: ["Breweries"],
    stages: ["Business core", "Business second release"],
    sections: [
      { heading: "Production board and operator view", intro: "Give each role a useful view of what needs attention today.", items: ["Production board organised by batch stage and tank", "Today task list for operators", "Role-based dashboard and saved views", "Operator display for production-floor iPads", "Daily tank rounds and printable weekly tank calendar"] },
      { heading: "Tank and batch movements", intro: "Model the physical journey of beer through the brewery.", items: ["Tank detail with occupancy, status, pressure limit and cleaning status", "Transfer between vessels with volume and loss records", "Split one batch across multiple tanks", "Merge batches with calculated blend properties", "Capacity conflict checks and tank scheduling"] },
      { heading: "Production records", intro: "Keep the planned process and actual work connected.", items: ["Operator identity, step completion and actual measurements", "Deviation records and corrective decisions", "Batch traceability from ingredients through packaging, with lot-to-batch recall reports", "Multiple operators and simultaneous active batches", "Audit trail for changes to closed production records", "Warnings when a tank without a valid cleaning record is assigned", "Shift handover of open alerts and production notes"] }
    ],
    note: "GRISTORA is not an ERP. Sales, orders, distribution, keg tracking at customer sites, keg deposits, CRM and accounting are outside the product scope."
  },
  {
    slug: "quality-laboratory",
    modules: ["quality"],
    title: "Quality, laboratory & release",
    eyebrow: "QUALITY MANAGEMENT",
    summary: "Professional quality workflows are built around product specifications, lab results, tasting and a recorded release decision before packaging.",
    audiences: ["Breweries"],
    stages: ["Business core", "Business second release", "Under consideration"],
    sections: [
      { heading: "Product specifications", intro: "Define measurable target ranges for each beer.", items: ["Recipe-linked specifications for ABV, OG, FG, IBU, colour and pH", "Dissolved oxygen, turbidity, diacetyl and microbiology parameters", "Out-of-range warnings and deviation records", "Quality review linked to the batch and its recipe version"] },
      { heading: "Laboratory and sensory records", intro: "Keep analytical and tasting results with the batch.", items: ["Laboratory results for pH, dissolved oxygen, CO₂, ABV and microbiology", "Tasting notes, sensory score and descriptive attributes", "Correction records rather than silent edits to historical results", "Sensory panel and triangle-test workflows"] },
      { heading: "Release and process control", intro: "Make the decision to package explicit and traceable.", items: ["Quality hold, rejection and release decisions with reasons", "Release approval before packaging in business workflows", "HACCP checkpoints and dual sign-off"] },
    ]
  },
  {
    slug: "packaging-compliance",
    modules: ["production","tax-compliance"],
    title: "Packaging, CIP & compliance",
    eyebrow: "PACKAGING AND RECORDS",
    summary: "Packaging records, vessel cleaning and jurisdiction-aware reporting support professional production, with clear boundaries around what the system does not do.",
    audiences: ["Breweries"],
    stages: ["Business core", "Business second release"],
    sections: [
      { heading: "Packaging runs and containers", intro: "Record what was packaged, in which format and how much.", items: ["Bottle, can, keg and UK cask container types with configurable sizes", "Packaging quantities, litres, losses, packaging efficiency and lot codes", "Multiple container types within one batch", "Bottle priming, keg pressure carbonation, canning from BBT and cask conditioning guidance", "Package material and empty weight for packaging reporting"] },
      { heading: "Cleaning and cellar process", intro: "Connect cleaning records to vessel readiness.", items: ["CIP chemical, concentration, temperature, duration and verification", "Tank cleaning status and last-cleaned record", "Warnings when a tank without a valid cleaning record is assigned", "Spunding and forced carbonation planning", "Cask conditioning and finings calculations for the UK market"] },
      { heading: "Labels and regulatory reports", intro: "Support production-side reporting without pretending to replace accounting or sales systems.", items: ["Country-specific tax calculation methods and reporting templates", "US TTB, UK HMRC, Australia ATO, Germany Zoll and Türkiye reporting scope", "ABV or Plato-based tax calculations according to jurisdiction", "Label information such as ABV, lot code, allergens and nutrition where requirements are confirmed", "Deposit-return and packaging/EPR reports"] }
    ],
    note: "Sales and shipment quantities may need manual entry or file import because sales and distribution are outside scope."
  },
  {
    slug: "team-notifications",
    modules: ["core","team"],
    title: "Teams, roles & notifications",
    eyebrow: "PEOPLE AND WORKFLOWS",
    summary: "One workspace supports a solo brewer and a multi-role production team, with notifications routed to the right person and actions recorded.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business core"],
    sections: [
      { heading: "Home and business workspaces", intro: "The workspace chooses Home or Business mode without splitting the underlying recipe and batch history.", items: ["Select a mode during onboarding and change it later in workspace settings", "Home mode keeps advanced business fields out of the way", "Optional invitations to collaborate on a recipe or batch in Home mode", "Switching modes preserves recipes and history; hidden business records are not deleted"] },
      { heading: "Team, roles and shared devices", intro: "Business workspaces support role-based production work.", items: ["Invite users with time-limited invitations and assign one or more roles", "Owner, brewmaster, operator, quality, warehouse and read-only auditor roles", "Custom roles, copied roles, permission groups and optional vessel/area scope", "Person-by-permission overview and role-specific opening screens", "Shared production iPad with personal PIN, fast user switching and automatic lock"] },
      { heading: "Notifications and approvals", intro: "A single notification record stays consistent across devices.", items: ["In-app inbox, push notifications and optional daily email summary", "Three priority levels, quiet hours, grouped alerts and notification history", "Acknowledge, snooze, ignore or resolve notifications with an event record", "Business shift routing, escalation and shift handover", "Recipe approval, quality release and other role-based approval queues"] }
    ]
  },
  {
    slug: "connected-devices",
    modules: ["devices","connections"],
    title: "Connected devices & integrations",
    eyebrow: "CONNECTED BREWING",
    summary: "Read-only fermentation device data sits alongside professional sensor and industrial gateway support.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business second release", "Later", "Under consideration"],
    sections: [
      { heading: "Homebrewing devices", intro: "Connect common fermentation monitors through dedicated adapters.", items: ["Tilt hydrometers, iSpindel and RAPT devices", "Webhook data endpoint for TiltBridge, Pitch and compatible bridges", "Device-specific keys that can be renewed", "Device-to-batch pairing and calibration records", "Show connection and reading status"] },
      { heading: "Professional sensors", intro: "Business mode adds tank-level monitoring.", items: ["Gravity, pH, dissolved oxygen and pressure readings", "Tank assignment and basic connection status", "API or CSV ingestion for professional sensors", "Industrial gateway read support for Modbus TCP, OPC UA and MQTT", "Sending control commands to equipment is outside scope"] },
      { heading: "Data use and reliability", intro: "Device data should remain useful when a sensor connection is interrupted.", items: ["Buffer sensor readings at the device or gateway and forward when connected", "Show source labels and calibration context alongside measurements", "Keep device adapters independent of any single manufacturer", "Use readings for fermentation trends and configured threshold alerts", "Open API and professional production-data webhooks for Business mode; Homebrew device webhooks are separate", "Industrial gateway support is read-only; equipment command and control is outside scope"] },
    ]
  },
  {
    slug: "imports-exports-tools",
    modules: ["core","connections"],
    title: "Import, export & brewing tools",
    eyebrow: "PORTABILITY AND UTILITIES",
    summary: "Bring existing recipe work into GRISTORA, move your data out when needed and use focused brewing tools alongside recipe and batch workflows.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Homebrew second release"],
    sections: [
      { heading: "Recipe portability", intro: "Import paths cover established brewing formats.", items: ["BeerXML and BeerJSON import/export", "BeerSmith .bsmx import", "Brewfather and Brewer's Friend recipe exports", "Ingredient, equipment, style and unit matching during import", "Show source-versus-GRISTORA calculation differences before saving"] },
      { heading: "Focused brewing tools", intro: "Focused utilities sit within the relevant recipe or batch context.", items: ["Refractometer correction and gravity tools", "Carbonation and priming calculators for bottles, kegs and casks", "Dilution, volume and unit conversion", "Extract brewing tools for partial boils, top-up water and LME/DME/honey conversions", "Decoction and parti-gyle tools; parti-gyle is available in Home mode", "No-chill IBU adjustment and bottle/keg carbonation tracking in Home mode", "Optional recipe transfer to supported brewing controllers in Home mode"] },
      { heading: "Reports and custom records", intro: "Make useful records portable and printable.", items: ["Brew sheets, tank rounds, fermentation and packaging forms", "Custom brew-sheet fields and layouts in Home mode", "Recipe, batch, equipment, water profile, ingredient, device and support data export", "CSV and JSON exports plus scanned documents and photos in downloadable archives", "Competition-form output and custom recipe/batch fields in Home mode", "Bring in laboratory water reports in Home mode"] }
    ]
  },
  {
    slug: "platform-security-data",
    modules: ["connections"],
    title: "Platform, security & data ownership",
    eyebrow: "PLATFORM",
    summary: "The product is one connected system across Apple devices and desktop web, with a shared server-side engine and explicit data ownership controls.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business core"],
    sections: [
      { heading: "Where GRISTORA runs", intro: "Each client has its own role.", items: ["iPad: full-featured native SwiftUI app", "iPhone: brew day, measurements, fermentation, stock and shopping workflows, with read-only recipe views; recipe design and planning stay on iPad and web", "Web: full-featured React and TypeScript experience for Mac and PC browsers", "Apple Watch is not supported", "Android and separate Mac/Windows desktop apps are not in scope"] },
      { heading: "One engine and one source of truth", intro: "The same calculation and business rules serve every client.", items: ["Rust calculation engine runs server-side; calculation code is not shipped to clients", "Shared API and data model across web and Apple applications", "Online-first operation with a limited downloaded brew-day package for offline timers and measurement entry", "No general offline synchronisation; queued brew-day measurements upload when connectivity returns", "Versioned calculation results and immutable recipe/batch references"] },
      { heading: "Security and data ownership", intro: "Security and portability are part of the platform.", items: ["Sign in with Apple or Google, with one-time email links as a fallback; no password-based signup", "Passkeys for business administrators, session management and remote sign-out", "Workspace separation, server-side permissions and append-only audit records", "Encrypted storage, protected file access and time-limited support access with customer consent", "Export your recipes, batches, measurements, stock, equipment and files; request account deletion"] },
      { heading: "Support and service", intro: "Customer support remains part of the product experience.", items: ["Searchable help and support request inbox", "Customer-visible request history and replies", "Optional, time-limited support access with a reason and audit history", "Internal support workspace separated from the customer application", "System status and support diagnostics"] },
      { heading: "Workspace setup, preferences and subscription", intro: "The first-run and account experience centres on a workspace rather than a separate registration form.", items: ["Guided first launch: choose Home or Business mode, data region and equipment profile", "Global search across recipes, batches, ingredients and stock", "Regional presets, category-specific units, local number/date formats and English/Turkish language support", "Theme, appearance, notifications, equipment, calculation and style-guide settings", "Subscription and entitlement management", "Account linking, session management, remote sign-out, account export and deletion"] },
      { heading: "Apple and web experience", intro: "System-level conveniences where the platform supports them.", items: ["iOS Live Activities, Dynamic Island, widgets, time-sensitive notifications and document scanning", "Siri and Shortcuts integration as part of the Apple experience", "Apple Watch is not supported; Android and separate desktop apps are out of scope", "Face ID or PIN app lock, shared-device PIN and administrator passkeys", "Encrypted backups, data portability and customer-visible support access history"] }
    ]
  },
  {
    slug: "analytics-reports",
    modules: ["cost-reports"],
    title: "Reports, insights & sustainability",
    eyebrow: "REPORTING AND INSIGHTS",
    summary: "Operational reporting covers batches, recipes and brewery production.",
    audiences: ["Breweries", "Homebrewers"],
    stages: ["Homebrew MVP", "Business core", "Business second release", "Under consideration"],
    sections: [
      { heading: "Batch and recipe learning", intro: "Use actual results to understand repeatability.", items: ["Plan-versus-actual batch summary and tasting notes", "Compare batches made from the same recipe", "Trends for gravity, efficiency and tasting scores in Home mode"] },
      { heading: "Brewery reporting", intro: "Business reporting focuses on production and material performance.", items: ["Brewhouse, conversion and lautering efficiency", "Volume loss from brew day through packaging", "FIFO ingredient cost and cost per litre", "Tank utilisation, batch throughput and production dashboard metrics", "Lot traceability and recall report from ingredient lot to packaging run"] }
    ]
  }
];

const seenFeatureSlugs = new Set<string>();
for (const feature of features) {
  if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(feature.slug)) {
    throw new Error(`Invalid feature slug: ${feature.slug}`);
  }
  if (seenFeatureSlugs.has(feature.slug)) {
    throw new Error(`Duplicate feature slug: ${feature.slug}`);
  }
  seenFeatureSlugs.add(feature.slug);
  if (feature.audiences.length === 0 || new Set(feature.audiences).size !== feature.audiences.length) {
    throw new Error(`Feature ${feature.slug} must have unique audience labels`);
  }
  if (feature.stages.length === 0 || new Set(feature.stages).size !== feature.stages.length) {
    throw new Error(`Feature ${feature.slug} must have unique roadmap stages`);
  }
  if (feature.sections.length === 0 || feature.sections.some((section) => !section.heading.trim() || section.items.length === 0)) {
    throw new Error(`Feature ${feature.slug} must have non-empty content sections`);
  }
}
