# G2 — Demleme name scan

Scan date: 2026-11-22  
Repository ref: `main` at `ccdf020a87820d6a084be0ff2ec1156f5d11be4d`

## Counts

| Path | User-visible matches |
|---|---:|
| `web/` | 2 |
| `ios/` | 2 |
| `sozlesme/` (catalog and OpenAPI) | 1 |
| `sunucu/` (email templates and auth UI defaults) | 1 |
| **Total** | **9** |

The scan records user-visible product-name uses only. It excludes code identifiers, file names, crate names, database names, test assertions, technical protocol strings, and uses of the common noun “demleme” (brewing), even when a label starts with a capital letter.

## Matches

| File | Line | Text | Kind |
|---|---:|---|---|
| `web/src/App.tsx` | 4 | `Demleme` (main page heading) | UI |
| `web/index.html` | 7 | `Demleme` (document title) | UI |
| `ios/Demleme/App/ContentView.swift` | 7 | `Demleme` (main screen heading) | UI |
| `ios/project.yml` | 33 | `Demleme` (iOS display name) | UI |
| `sunucu/crates/api/src/auth/config.rs` | 216 | `Demleme` (default WebAuthn relying-party name in the passkey dialog) | UI |
| `sozlesme/openapi/openapi.json` | 13734 | `Demleme` (passkey contract example) | UI |
| `arayuz/demleme-arayuz-konsepti.html` | 1 | `Demleme Arayüz Konsepti` (document title) | Concept |
| `arayuz/demleme-arayuz-konsepti.html` | 447 | `Demleme Arayüz Konsepti` (page heading) | Concept |
| `arayuz/demleme-arayuz-konsepti.html` | 3800 | `Demleme Uygulaması Ürün Planı` (linked document title) | Concept |

## No-match paths

- `sozlesme/katalog/katalog.json`: no match.
- `sunucu/crates/common/src/catalog/katalog.json`: no match.
- `sunucu/crates/api/src/auth/email.rs`: the match is an SMTP greeting string (`EHLO demleme.local`), not user-visible email content; excluded.
## Limits

This is a text-source scan at the recorded `main` commit. It does not inspect rendered screenshots or external runtime content.
