# web

React + TypeScript (Vite) web uygulaması. İskelet: iş T0.5. Kod İngilizce, belgeler Türkçe (ADR-0001).

## Gereksinimler

Node 22 (`.nvmrc`), npm. Chromium e2e testi için sistemde `/opt/pw-browsers/chromium` adresinde kurulu olmalıdır; başka bir yol için `PW_CHROMIUM_PATH` ortam değişkeni kullanılır. `playwright install` çalıştırılmaz.

## Komutlar

| Komut                  | Ne yapar                                                      |
| ---------------------- | ------------------------------------------------------------- |
| `npm ci`               | Bağımlılıkları `package-lock.json`'a göre kurar               |
| `npm run dev`          | Geliştirme sunucusu                                           |
| `npm run build`        | Tip denetimi ve üretim derlemesi (`dist/`)                    |
| `npm run typecheck`    | `tsc` ile tip denetimi (`strict`, `noUncheckedIndexedAccess`) |
| `npm run lint`         | ESLint                                                        |
| `npm run format:check` | Prettier denetimi (`npm run format` düzeltir)                 |
| `npm test`             | Vitest birim testleri                                         |
| `npm run e2e`          | Playwright duman testi (derler, `vite preview` başlatır)      |
| `npm run gen:tokens`   | `src/styles/tokens.css` dosyasını yeniden üretir              |
| `npm run check:tokens` | Üretilen dosya güncel değilse hata verir                      |

## Tasarım token'ları

`tasarim-sistemi/tokens.json` asıl kaynaktır; `scripts/gen-tokens.mjs` ondan `src/styles/tokens.css` üretir. Üretilen dosya depoda tutulur ve elle düzenlenmez. Token değişince `npm run gen:tokens` çalıştırılır ve dosya commit'e girer.

- CSS değişkeni adı token adıyla aynıdır: `bg` → `var(--bg)`. Yazı ailesi `--font-sys`, `--font-round`, `--font-mono`; metin stili `--t-body-size`, `--t-body-line`, `--t-body-weight`.
- Açık tema `:root`'ta. Koyu tema `prefers-color-scheme: dark` altında ve `[data-theme="dark"]` ile. Yüksek karşıtlık yalnız birkaç token'ı açık temanın üstünde değiştirir; `prefers-contrast: more` altında (açık sistem temasında) ve `[data-theme="contrast"]` ile devreye girer. `[data-theme="light"]` sistem ayarını geçersiz kılar.
