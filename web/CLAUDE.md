# web

## Ne yapar

- React + TypeScript (Vite) web arayüzü; TypeScript `strict` ve `noUncheckedIndexedAccess` açık.
- Renk, yazı ve boşluk token'ları `tasarim-sistemi/tokens.json`'dan üretilir; açık, koyu ve yüksek karşıtlık temaları CSS değişkenidir.
- Node 22 (`.nvmrc`), npm.

## Neye dokunmaz

- İş kuralı ve hesap yok; hesap sunucudadır (`sunucu/`), web yalnız arayüzü çizer.
- Token değerleri `tasarim-sistemi/tokens.json`'da değişir, `src/styles/tokens.css`'de değil.

## Nasıl test edilir

- `npm ci`, ardından `npm run typecheck`, `npm run lint`, `npm run format:check`, `npm test`.
- `npm run check:tokens`: üretilen token dosyası güncel mi.
- `npm run e2e`: Chromium `/opt/pw-browsers/chromium` adresinde olmalı (ya da `PW_CHROMIUM_PATH`); `playwright install` çalıştırılmaz.

## Kurallar

- `src/styles/tokens.css` üretilir (`npm run gen:tokens`); elle düzenlenmez.
- Kod İngilizce; belgeler Türkçe.
- Kullanıcıya görünen metin kodda yazılmaz; metin kataloğundan gelir (Türkçe ve İngilizce).
- Commit mesajı iş numarasıyla başlar: `T0.x: ...`.
