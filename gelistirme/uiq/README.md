# UIQ: UI quality program for web and iPad

Decision record: `adr/0016-arayuz-kalite-programi.md`. Owner request: Oğuz, 2026-11-27. This file uses ASD-STE100. Read the section that your task names. Do not read all of this file for one task.

## 1. Scope

- Clients: the product web app (`web/`) and the iPad app (`ios/`). Not the marketing site (`marketing-site/`).
- In scope: the GRISTORA React UI Kit (`web/src/kit/`), the GRISTORA SwiftUI Kit (`ios/Demleme/Kit/`), structural tokens, adaptive layout, the Visual Quality Gate in CI, and the migration of the current screens.
- Out of scope: iPhone and iPad mini optimization, 4K, ultrawide, old browsers, Android. Do not delete iPhone code, targets or tests (ADR-0016 › 6).
- Do not change the Rust server, the engine, the OpenAPI contract or business rules. Do not put formulas in a client.
- Do not add a dependency other than `react-aria-components@1.22.1`. Another dependency needs a Claude architecture approval.
- Forbidden packages: Material UI, Ant Design, Bootstrap, Tailwind CSS, Radix UI, Base UI, Storybook, TanStack Table, ECharts, Visx, Floating UI (direct), Swift Snapshot Testing, Flutter, React Native.

## 2. Apple patterns for GRISTORA (UIQ-02)

| Apple app | Pattern | GRISTORA use | Kit parts |
|---|---|---|---|
| Numbers | Sidebar, canvas, Inspector at the right; table with a sticky header | Recipe editor, ingredient tables, water and mash panels | `AppShell`, `InspectorPanel`, `DataTable`, `Toolbar` |
| Health | Metric card with value, unit, time and trend; detail chart with ranges | Batch measurements, gravity and temperature history | `SummaryCard`, `MeasurementCard`, `ChartContainer` |
| Calendar | Day, week and month; time bands; color only as a category | Production calendar, tank plan (stage 6) | `Timeline` (stage 6), `SegmentedControl` |
| Reminders | Check list with sections; a row completes with one tap | Brew day steps, CIP check lists | `GroupedList`, `Checkbox` |
| Settings | Grouped form with sections and footers; navigation rows | Settings, modules, workspace, members | `GroupedList`, `FormField`, `Toggle` |
| Notes | List, then content; clear hierarchy of title and body | Batch notes, tasting notes | `SplitView`, `PageHeader` |
| Files | Browse, search, sort; list and grid views | Attachments, imports | `SearchField`, `DataTable`, `StateView` |

Rules:
- Use the pattern, not the look. Do not copy Apple artwork, icons or brand elements.
- The operator view (ISA-101) wins over all patterns on the production board, Today, the tank detail and brew day. Normal state is gray. Color shows only a deviation.
- Do not use glass on content, tables or the operator view.

## 3. Token standard (UIQ-03)

The single source is `tasarim-sistemi/tokens.json`. Do not edit generated files (`web/src/styles/tokens.css`, `ios/Demleme/Design/Tokens.swift`, the color assets).

| Group | Tokens | Rule |
|---|---|---|
| `color` | existing | Do not add colors without a contrast check (4.5:1 text, 3:1 non-text). |
| `spacing` | `space-1`…`space-8` | Use only these values for gap, padding and margin. |
| `radius` | `radius-xs`…`radius-pill` | As in the design system README. |
| `size` | `tap-min`, `control-height`, `control-height-sm`, `row-regular`, `row-compact`, `row-relaxed`, `icon-sm`, `icon-md`, `badge-size`, `focus-ring-width`, `focus-ring-offset` | `control-height-sm` and `row-compact` are for web with a pointer only. Touch targets are at least `tap-min`. |
| `layout` | `sidebar-*`, `inspector-*`, `content-min`, `form-max`, `content-max`, `bp-sidebar`, `bp-inspector` | `bp-*` is the sum of the panel widths. A test checks the sums. |
| `z` | `z-base`…`z-skip-link` | The only z-index values. |
| `motion` | `motion-fast`, `motion-base` | 0 ms when reduced motion is on. |
| chart palette | `chart-1` … `chart-6` | Series colors are not status colors. Each series also has a fixed marker (circle, square, triangle, diamond, cross, dashed line) and a label. Contrast against `card` is 4.5:1 or more in the four themes. |

Density: the web has `data-density="regular|compact"` on a table or a list. Compact uses `row-compact`. The iPad uses only regular and relaxed.

## 4. Component standard

### 4.1 Rules for all components

1. Size: height is at least `control-height` for a control. Width has `min-inline-size` from the content and no fixed `inline-size`, unless the table says so.
2. Long text: a label wraps to two lines, then truncates with an ellipsis and a full-text tooltip (pointer) or the full text in the accessible name. A number never truncates. Use `tabular-nums` for values.
3. Narrow space: use a container query. A row goes to a column. A toolbar moves extra actions to an overflow menu ("More").
4. Themes: light, dark, high contrast, dark high contrast. Use only tokens. Test all four.
5. States: normal, hover (pointer only), pressed, selected, focus-visible, disabled, invalid, pending (loading). Use the `data-*` attributes of React Aria for styles.
6. Focus: ring `blue`, `focus-ring-width`, `focus-ring-offset` outside. Never remove the ring.
7. Accessibility: a name for each control; a status always has a word or an icon with a label; `aria-live="polite"` for feedback; keyboard order follows the reading order.
8. Text: all visible text comes from the text catalog (TR and EN). EN text is often shorter than TR; test TR for width.

### 4.2 Component inventory

"Base" is the React Aria part (web) and the system part (iOS). "DS" is the existing design system component in `tasarim-sistemi/components/`.

| Kit name | Includes (owner names) | Web base | iOS base | DS | Notes |
|---|---|---|---|---|---|
| `AppShell` | AppShell, SplitView | own grid | `TabView(.sidebarAdaptable)`, `NavigationSplitView` | NavBar | Sidebar + Content + Inspector |
| `Sidebar` | Sidebar | `GridList` or nav list | system sidebar | – | Selected row `tint-soft`; module state marks |
| `InspectorPanel` | InspectorPanel | own; `Modal` as sheet below `bp-inspector` | `.inspector` | – | `inspector-min`…`inspector-max` |
| `PageHeader` | PageHeader, Breadcrumb | `Breadcrumbs` | `navigationTitle`, toolbar | NavBar | Breadcrumb only for depth > 3 (web) |
| `SectionHeader` | SectionHeader | `Header` | `Section` header | GroupedList | – |
| `Toolbar` | Toolbar | `Toolbar` | `.toolbar` | – | Overflow to "More" menu |
| `Tabs` | Tabs | `Tabs` | `TabView` page or `Picker` | – | Web only for in-page sections |
| `SegmentedControl` | SegmentedControl | `ToggleButtonGroup` | `Picker(.segmented)` | SegmentedControl | 2–5 segments |
| `SummaryCard` | MetricCard, SummaryCard | own | own | SummaryCard | `t-metric`; no shadow |
| `MeasurementCard` | MeasurementCard | own (SummaryCard + chart slot) | own + Swift Charts | – | Value, unit, time, trend |
| `DataTable` | DataTable, DataRow | `Table` | `Table` | – | Sticky header, sort, numeric columns end-aligned |
| `GroupedList` | DataRow (Settings style) | `GridList`/`ListBox` | `List(.insetGrouped)` | GroupedList | – |
| `StatusBadge` | StatusIndicator, StatusBadge | own | own | – | Word + icon, never color only |
| `AlarmMark` | AlarmMark | own | own | AlarmMark | ISA-101 shapes 1–3 |
| `ProgressIndicator` | ProgressIndicator | `ProgressBar`, `Meter` | `ProgressView`, `Gauge` | – | – |
| `ChartContainer` | ChartContainer | own SVG frame | Swift Charts | – | Title, text summary, table alternative |
| `Timeline` | Timeline | own | own | – | Stage 6 (calendar, tank plan) |
| `FormField` | FormField | `Label`, `Text slot`, `FieldError` | `LabeledContent`, `Form` | – | Label, help, error |
| `TextField` | TextField | `TextField` | `TextField` | – | – |
| `NumberField` | NumberField, UnitInput | `NumberField` + unit suffix | `TextField(value:format:)` | – | The unit comes from the server catalog. No conversion in the client. |
| `SelectField` | SelectField | `Select` | `Picker` | – | – |
| `ComboBox` | ComboBox | `ComboBox` | `Picker` + search, or `.searchable` | – | – |
| `DatePicker` | DatePicker | `DatePicker` | `DatePicker` | – | Locale format |
| `SearchField` | SearchField | `SearchField` | `.searchable` | – | – |
| `Toggle` | Toggle | `Switch` | `Toggle` | Toggle | – |
| `Checkbox` | Checkbox | `Checkbox` | `Toggle(.checkbox)` style or list check | – | – |
| `Button` | PrimaryButton, SecondaryButton, IconButton | `Button` | `Button` + `ButtonStyle` | Button | Variants `primary`, `secondary`, `destructive`, `plain`; one primary for each screen; icon-only needs a name |
| `Menu` | Menu | `MenuTrigger`, `Menu` | `Menu` | – | – |
| `Popover` | Popover | `Popover` | `.popover` | – | Flips at the viewport edge |
| `Tooltip` | Tooltip | `TooltipTrigger` | `.help` | – | Pointer only; never the only source of information |
| `Dialog` | Dialog, Sheet, ConfirmationDialog | `Modal`, `Dialog` | `.sheet`, `.confirmationDialog`, `.alert` | – | No sheet on a sheet |
| `StateView` | EmptyState, LoadingState, ErrorState | own | `ContentUnavailableView`, `ProgressView` | – | `kind`: empty, loading, error |
| `Callout` | WarningBanner, SuccessFeedback | own + live region | own | Callout | Tones: info, success, warning, error |
| `BrewReturnBar` | – | own | own | BrewReturnBar | Visible on each screen during a brew |

Domain parts are not Kit parts. Build `TankCard` (stage 6) and the calendar bands from Kit parts in the feature code.

Duplicates in the current code (to merge in UIQ-13):
- web `.module-message`, `.module-banner`, `.module-notice` → `Callout`.
- web theme `<select>` in the header → `SelectField`; later the theme choice moves to Settings.
- iOS `ModuleNoticeView` → `Callout`; `ModuleWriteButton` → `Button` with a disabled reason.

## 5. Web adaptive layout standard (UIQ-06)

- Use CSS Grid and Flexbox, `minmax(0, 1fr)`, `min-inline-size: 0` on grid and flex children, logical properties (`inline`, `block`), `flex-wrap`, `clamp()` for page gutters.
- Only `AppShell` reads the viewport. Each other component uses `container-type: inline-size` and `@container`.
- Breakpoints of `AppShell` (values from tokens; a container query cannot read `var()`, so the CSS repeats the value and a unit test compares it with `tokens.json`):
  - width ≥ 1060 px: sidebar, content and inspector are columns.
  - 740–1059 px: the inspector is a sheet from the inline end.
  - width < 740 px: the sidebar is also an overlay.
- Do not use `overflow: hidden` to hide a layout error. Scroll only in a scroll region. Mark it with `data-scroll-region`.
- Do not use fixed heights for content. Use `min-block-size`.
- Use only `z` tokens. Overlays come from React Aria and go to the top layer.
- `web/scripts/check-css.mjs` fails when CSS outside `web/src/kit/` and `tokens.css` has a raw color, a raw `z-index` or `overflow: hidden` without the comment `/* allowed: reason */`.

## 6. iPad adaptive layout standard (UIQ-07)

- Root: `TabView` with `.tabViewStyle(.sidebarAdaptable)` (iPadOS 18). The iPhone keeps the tab bar.
- List and detail: `NavigationSplitView`. Helper panel: `.inspector` with `inspectorColumnWidth(min:ideal:max:)` from `DemlemeMetric`.
- Landscape: sidebar, content and inspector are visible. Portrait: content first; the inspector opens as a sheet when the width is compact.
- Keep the selection and edits through a size change: keep them in `@SceneStorage` or the store, not in a view `@State` that a layout change resets.
- Use `ViewThatFits`, `Layout`, `LazyVGrid(.adaptive)`, `ScaledMetric`, `safeAreaInset`, `ContentUnavailableView`, Dynamic Type, SF Symbols, Swift Charts.
- Do not use fixed device widths, `offset` for layout, absolute positions or fixed content heights.
- Each Kit view has a `#Preview` for light, dark, increased contrast, the largest Dynamic Type size and a long-text case.

## 7. Visual Quality Gate (UIQ-10, UIQ-11, UIQ-12)

### 7.1 Web

- Surfaces: the Kit gallery (`web/kit.html`, `vite build --mode kit`, output `dist-kit/`, not in the product build) and the real screens with API mocks.
- Matrix: 7 viewports (820×1180, 1180×820, 1024×1366, 1366×1024, 1280×720, 1920×1080, 2560×1440) × 4 themes × 2 languages. Screenshot comparison uses a smaller matrix: 1366×1024 and 820×1180, light and dark, EN. Geometry and axe run on all cells.
- Checks:
  - overlap: visible interactive elements do not intersect, except the allowed overlays;
  - overflow: no horizontal page scroll; no element outside the viewport;
  - clipping: no text with `scrollWidth > clientWidth` outside a scroll region or an ellipsis that has a full name;
  - alignment: the edges of the main columns and the table cells align within 1 px;
  - axe: tags `wcag2a`, `wcag2aa`, `wcag21aa`, `wcag22aa`;
  - reflow: 320 CSS px and 640 CSS px wide (400% and 200% zoom of 1280 px).
- Browsers: Chromium (Chrome, Edge) and Playwright WebKit (Safari).
- Fixed environment: `ubuntu-24.04`, the Playwright version from `package-lock.json`, `deviceScaleFactor: 1`, `reducedMotion: 'reduce'`, a fixed clock, `animations: 'disabled'`.
- Baselines: only a `workflow_dispatch` job makes them, as a build artifact. Claude reviews the images and commits them. Do not use `--update-snapshots` in a normal run. Baseline files are class C.

### 7.2 iPad

- Simulators: iPad Pro 11-inch and iPad Pro 13-inch, landscape and portrait (`XCUIDevice.shared.orientation`). The Xcode and iOS runtime versions are fixed in `ios.yml`.
- Checks: `XCUIScreenshot` with `XCTAttachment`, a local pixel compare helper with a threshold, `performAccessibilityAudit()`, element frames inside the window, no overlap of hittable elements.
- Gallery: a DEBUG launch argument `-KitGallery` opens a screen with each Kit view.
- Baselines: in `ios/DemlemeUITests/Baselines/<device>-<orientation>/`. Only a CI run makes them. Claude reviews and commits them.

### 7.3 Rollout

1. Report mode: the job runs and shows the result. It is not a required check.
2. After two stable weeks, the owner makes it a required check in branch protection.
3. Do not change the current security and merge rules.

## 8. Acceptance criteria for a critical screen

- No unwanted overlap. No unwanted overflow. No clipped value or label.
- Buttons and fields are at least `tap-min` (touch) and have names.
- Tables align and stay readable at `row-compact` and `row-regular`.
- Menus and popovers stay in the viewport.
- The four themes are consistent. TR and EN fit.
- A long record name does not break the layout.
- Empty, dense, loading and error states are correct.
- The keyboard and accessibility checks pass. WCAG 2.2 AA.
- Apple HIG patterns and the ISA-101 operator view are kept.
- A manual review by Claude is done. Automatic tests do not replace it.

## 9. Tasks

The order is in the dependency column. Tasks that change the same files are in a chain. Class: the permission class of the main paths (`marketing-site/CHATGPT-SCOPE.md` › Permission classes).

| Task | Description | Done criterion | Depends on | Class | Owner | Model | Status |
|---|---|---|---|---|---|---|---|
| UIQ-01 | Visual audit of the current web and iPad screens | `gelistirme/raporlar/UIQ-01-gorsel-denetim.md` | – | A | Claude | Sonnet (scan), Opus (review) | done (2026-11-27) |
| UIQ-02 | Apple patterns mapped to the design system | Section 2 of this file | – | A | Claude | Opus | done (2026-11-27) |
| UIQ-03a | Structural tokens (`size`, `layout`, `z`, `motion`) for CSS and Swift | Generators `--check` pass; token sum tests pass | – | C | Claude | Sonnet | done (2026-11-27) |
| UIQ-03b | Chart palette in four themes; design system README and Artifact updated | 3:1 against `card` in each theme; `araclar/tutarlilik.py` clean | UIQ-03a | C | Claude | Opus | done (2026-11-28) |
| UIQ-04 | React UI Kit skeleton: `web/src/kit/`, gallery entry `kit.html`, `Callout`, `StateView`, `SummaryCard`, `StatusBadge`, `AlarmMark` (no React Aria) | vitest and RTL for each part; gallery builds; axe clean | UIQ-03a | B | ChatGPT | – | todo |
| UIQ-05 | SwiftUI Kit skeleton: `ios/Demleme/Kit/`, button styles, `SummaryCard`, `StatusBadge`, `AlarmMark`, `Callout`; previews | macOS CI green; previews in four themes | UIQ-03a | C | Claude | Sonnet | done (2026-11-27) |
| UIQ-06 | Web adaptive `AppShell` (Sidebar + Content + Inspector), `check-css.mjs` | Geometry test at 7 viewports; no horizontal scroll | UIQ-04 | B | ChatGPT | – | todo |
| UIQ-07 | iPad adaptive root: `.sidebarAdaptable`, split view, inspector, state kept through rotation | UI test rotates and keeps the selection; iPhone UI test passes | UIQ-05 | C | Claude | Sonnet | todo |
| UIQ-08 | React Aria pilot: dependency, `Button`, `Toggle`, `SelectField`, `Dialog`; ESLint import rule | Pilot criteria in ADR-0016 › 2 | UIQ-04 | B + dependency (Claude review) | ChatGPT | – | todo |
| UIQ-09 | React Aria rollout: form fields, `Menu`, `Popover`, `Tooltip`, `DataTable`, `SegmentedControl`, `Tabs`, `Toolbar` | Tests for each part; gallery; axe clean | UIQ-08 | B | ChatGPT | – | todo |
| UIQ-10 | Web geometry and screenshot tests on the gallery and the screens | Specs in `web/e2e/visual/`; run local in report mode | UIQ-04 | A | ChatGPT | – | todo |
| UIQ-11 | iPad screenshot and accessibility tests; compare helper; `-KitGallery` | Attachments in the CI run; audit passes | UIQ-05, UIQ-07 | C | Claude | Sonnet | todo |
| UIQ-12 | CI Visual Quality Gate: `visual` job, WebKit, baseline job; iOS iPad job | Jobs green in report mode; first baselines reviewed and committed | UIQ-10, UIQ-11 | C | Claude | Sonnet | todo |
| UIQ-13 | Move the current screens to the Kit (web shell, containers, modules; iOS modules, scan) | No duplicate listed in section 4.2 remains; gate green | UIQ-06, UIQ-09, UIQ-07 | B (web), C (iOS) | ChatGPT (web), Claude (iOS) | Sonnet (iOS) | todo |
| UIQ-14 | Theme, language and accessibility audit (TR, EN, four themes, Dynamic Type, zoom) | Report with findings as issues | UIQ-13 | A | ChatGPT | – | todo |
| UIQ-15 | Fix the found visual defects | Each issue from UIQ-14 closed or decided | UIQ-14 | B, C | ChatGPT, Claude | Sonnet | todo |
| UIQ-16 | Final visual quality and performance review (bundle size, LCP, INP; iPad scroll) | Report `gelistirme/raporlar/UIQ-16-*.md`; Claude manual review | UIQ-15, UIQ-12 | A | Claude | Opus | todo |

Parallel work:
- UIQ-04 (web) and UIQ-05 (iOS) are independent after UIQ-03a.
- UIQ-06, UIQ-08 and UIQ-10 all change `web/`. Give them to ChatGPT in this order: UIQ-08 after UIQ-04, then UIQ-06; UIQ-10 can go in parallel because it changes only `web/e2e/visual/`.
- Do not change `web/src/shell/` in UIQ-04 or UIQ-08. UIQ-06 owns it.
