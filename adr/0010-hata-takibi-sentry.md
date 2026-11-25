# ADR-0010: Hata ve çökme takibi: Sentry

**Durum:** kabul · **Tarih:** 24 Kasım 2026 · **Karar:** Oğuz (24.11.2026) · **İş:** O9

## Bağlam

Gözlem OpenTelemetry ile kuruldu (plan › Ortamlar ve işletim; ADR-0008 madde izleme). Log, metrik ve iz Google Cloud'a (Cloud Logging, Monitoring, Trace) gider. Bunlar sunucu tarafı için yeterli, ama iki boşluk var:

- iPhone, iPad ve web istemcilerinde çökme ve hata raporu yok (sembolleştirilmiş yığın izi, sürüm, cihaz).
- Sunucu hatalarını gruplayan, yeni hatada ve gerilemede (regression) uyaran, sürüme bağlayan bir araç yok.

Değerlendirilenler: Sentry, Google Cloud Error Reporting (Google'a özgü; taşınmayı kod değişikliğine çevirir), Crashlytics (Firebase; plan Firebase'i kullanmıyor), kendi sunucumuzda GlitchTip (işletim yükü).

## Karar

1. **Araç:** Sentry (SaaS). Hata ve çökme takibi için tek araç; sunucu, web ve iOS aynı projede, sürüm etiketiyle.
2. **Görev ayrımı:** iz, metrik ve log OpenTelemetry'de kalır. Sentry yalnız hata, çökme ve sürüm sağlığını alır. Sentry olayı `trace_id` taşır; böylece hata, Cloud Trace'teki ize bağlanır.
3. **İstemciler:**
   - Sunucu (Rust; api, worker, ingest, hesap motoru): `sentry` ve `sentry-tracing`; `tracing` hata olayları Sentry'ye gider.
   - Web (React): `@sentry/react`; kaynak haritası (source map) CI'da yüklenir, yayına konmaz.
   - iOS: `sentry-cocoa`; çökme raporu ve dSYM yüklemesi macOS CI'da.
4. **Gizlilik:**
   - `send_default_pii` kapalı. IP adresi saklanmaz. E-posta, ad, tarif metni, ölçüm değeri ve istek gövdesi gönderilmez.
   - Kullanıcı kimliği yerine rastgele, geri çevrilemeyen bir kimlik gider (hesap kimliğinin anahtarlı özeti).
   - Oturum tekrarı (session replay) ve ekran görüntüsü kapalı.
   - Sunucu tarafında `before_send` süzgeci başlık ve sorgu parametrelerindeki belirteçleri siler; bunun testi yazılır.
5. **Veri bölgesi:** Sentry'nin ABD ve AB (Frankfurt) bölgeleri var \[doğrulanacak\]. İngiltere kullanıcıları için UK GDPR açısından AB bölgesi tercih edilir; son seçim gizlilik politikası yazılırken yapılır. Sentry alt işleyen olarak gizlilik politikasına ve veri işleme sözleşmesine eklenir.
6. **Örnekleme ve kota:** hatalar yüzde yüz; Sentry'de performans izi (tracing) kapalı, çünkü iz OpenTelemetry'de. Aylık kota ve plan beta öncesi seçilir \[doğrulanacak\].
7. **Ortam ve anahtar:** DSN sır yöneticisinden okunur (`SENTRY_DSN` / `SENTRY_DSN_FILE`). DSN yoksa Sentry kapalıdır; yerel geliştirme, compose ve kind Sentry'siz çalışır.
8. **Çıkış yolu:** Sentry SDK'ları GlitchTip ile uyumludur \[doğrulanacak\]. Gerekirse yalnız DSN değişir, kod değişmez.

## Sonuçlar

- Yeni bağımlılıklar (Rust `sentry`, `@sentry/react`, `sentry-cocoa`) Claude incelemesinden geçer (yetki sınıfı kuralı).
- İşler ilgili aşamanın listesine yazılır: web (aşama 3), iOS ve dSYM yüklemesi (aşama 4), sunucu ve canlı DSN (aşama 5, bulut kurulumu).
- Uyarılar: yeni hata türü, gerileme ve hesap motoru hata oranı. Hesap motoru süre alarmı OpenTelemetry metriğinde kalır.
- Gizlilik politikası ve alt işleyen listesi güncellenir.
