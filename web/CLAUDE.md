# web

## What it does

- React + TypeScript (Vite) web interface. TypeScript options `strict` and `noUncheckedIndexedAccess` are on.
- Tokens for color, type and space come from `tasarim-sistemi/tokens.json`. The light, dark and high-contrast themes are CSS variables.
- Node 22 (`.nvmrc`), npm.

## What it does not touch

- It has no business rules and no calculation. The server (`sunucu/`) does the calculation. The web app only draws the interface.
- Change token values in `tasarim-sistemi/tokens.json`. Do not change them in `src/styles/tokens.css`.

## How to test

- Run `npm ci`. Then run `npm run typecheck`, `npm run lint`, `npm run format:check` and `npm test`.
- `npm run check:tokens`: checks that the generated token file is current.
- `npm run e2e`: Chromium must be at `/opt/pw-browsers/chromium` (or set `PW_CHROMIUM_PATH`). Do not run `playwright install`.

## Rules

- `npm run gen:tokens` generates `src/styles/tokens.css`. Do not edit it by hand.
- Code is in English. Notes for Claude (this file, task lists) use ASD-STE100. Other documents are in Turkish.
- Do not write user-visible text in code. Text comes from the text catalog (Turkish and English).
- Start the commit message with the task number: `T0.x: ...`.
