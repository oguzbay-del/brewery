# ADR-0014: Aşama 2 kapanışı (çekirdek API ve veri)

**Durum:** kabul · **Tarih:** 27 Kasım 2026 · **İş:** T2.35

## Kapı

| Kapı maddesi | Sonuç |
|---|---|
| Uçtan uca testler compose ve kind'da yeşil | `make e2e` compose yığınında 11 senaryo dosyası (`sunucu/crates/e2e/tests/s1`–`s11`: Home akışı, iki çalışma alanının yalıtımı, eksik oturum ve izin, eski sürüm, paket imzası ve anahtarı, düzenleme oturumu olayları, idempotency ve ETag, fıçılar, modüller). `altyapi.yml` her PR'da compose duman testi + `make e2e`, kind duman testi (Gateway, oauth2-proxy, Istio ambient mTLS, ağ ve yetki politikaları, `/internal` dışarıdan 404) çalıştırır. Main'de son koşu yeşil (`3fca982`, 27.11.2026) |
| F2.1 ve F2.2'nin kritik ve önemli bulguları kapandı | F2.1: kritik yok; 13 önemli bulgunun 12'si T2.9'da düzeltildi (göç 0100–0102), 11. bulgu kısmen (saklama süreleri `common::retention`'da, mevzuat doğrulanacak; karar 10 açık, aşağıda). F2.2: kritik yok; 3 önemli bulgu (G1 App Store webhook, G2 ve G3 step-up) T2.34'te, kalan step-up eylemleri T2.53'te kapandı |
| Birleştirme kapısı yeşil | Main'de `sunucu` (fmt, clippy `-D warnings`, test, sqlx denetimi), `sozlesme` (OpenAPI ve TS tipleri güncel, kırıcı fark denetimi), `belgeler` (`araclar/tutarlilik.py`), `web`, `ios`, `altyapi` iş akışları yeşil. Korumalı yol ve bağımlılık kapıları (#256) çalışıyor |

## Sapmalar ve kararlar

- **Kapsam İşletme modüllerinden parça içerir:** fıçı ve kap takibi (K1, K2, göç 0250 ve 0252) ve ürün modülleri (P1) Aşama 2'de yapıldı. Sahip kararıyla (20.11.2026 ve 25.11.2026) öne alındı; Aşama 6'nın İşletme ekranları değişmedi.
- **İş dağılımı:** Aşama 2'nin çoğu işi ChatGPT tarafından yazıldı, Claude inceledi ve birleştirdi (sahip kararı, 22.11.2026; döngü v2, ADR-0011). C sınıfı yollar Claude incelemesinden geçti.
- **Ödeme:** Paddle adaptörü yazılmadı. Production'da web ödeme yolu 503 verir. Plan ve koltuk modeli (B2–B4, B7) #390 zincirinde.
- **Hesap motorunun dışında kalan iki hesap:** `est_cells_ml` ve `honey_pct` API'de hesaplanıyor; verim kalibrasyonu kuralı da API'de (T2.20, T2.21). Motora taşınacak (`TODO` notları kodda). İstemciye hesap kodu girmedi.

## Aşama 3'e taşınan açık işler (kapanışa engel değil)

- **F2.2 küçük bulguları:** G5 cihaz ucu hız sınırı (T2.54, #395), G7 destek aramasında personel denetimi (T2.55, #396, göç 0253). G4 (iki sertifikalı App Store zincirinde ara sertifika işareti) açık, düşük önemde (T2.60, #413). G6 sahip kararıyla kabul edildi (GitHub düzenleme geçmişi riski).
- **T2.58** (#403): deneme bitişi ve salt okunur modül E2E senaryosu; B2'den sonra.
- **T2.59** (#409, PR #412): paket tipi API'si; K3 ve K4 ekranları bunu kullanır.
- **F2.1 karar 10:** yasal tutmalı kayıtların çalışma alanı silmesindeki davranışı; süreler doğrulanacak.
- **Bilinen sınırlar:** ztunnel yalnız L4 uygular, `/internal` yol kuralı için waypoint eklenmedi (HTTPRoute ve `INTERNAL_PROXY_TOKEN` kalır); Apple `invalid_grant` davranışı gerçek servisle doğrulanacak; ölçüm bölümleri ve `validate_measured_at` hazır.

## Sonuçlar

- Aşama 3 (web, Home MVP ekranları) başlar. Kapısı: Playwright ve erişilebilirlik testleri. Opus Aşama 3 iş listesini ISLER'e yazar; mevcut T3.a–T3.d ve P3 (#326 birleşti, #392) bu listeye girer.
- Bu ADR, `adr/` numara sırasında 0014'ü alır. MCP ve OAuth taslağı (#389) 0015'i kullanır.
