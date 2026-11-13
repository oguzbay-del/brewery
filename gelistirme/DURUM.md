# Durum

Her oturum önce bu dosyayı okur. En fazla bir sayfa. Her iş bitince güncellenir.

**Aşama:** 2 Çekirdek API ve veri (Aşama 1 kapandı: ADR-0006)
**Sıradaki iş:** T2.7 İşletme göçleri alt ajanda sürüyor; ardından T2.11 giriş, T2.14 denetim kaydı, T2.15 olaylar, T2.16 motor bağlantısı, T2.18 nesne deposu (hepsi T2.10'a bağlı, paralel yürüyebilir). Aşama 2 listesi ISLER.md'de (T2.0–T2.35, F2.1, F2.2).
**Yöntem:** ADR-0003 (Python → JSON vektör → Rust parite testi; `make vektor`).

## Ortam

- Kurulu: Rust 1.97, Node 22, Docker istemcisi. Oturum başında `dockerd` elle başlatılır.
- scratchpad/bin: terraform 1.17.2, kind 0.33.0, kubectl 1.37.2, actionlint 1.7.12. Yeni oturumda scratchpad boşsa yeniden indirilir (README'lerde yazılı).
- Testler PostgreSQL ister: `docker compose -f altyapi/compose/compose.yaml up -d --wait postgres` ya da `make up`; `make test` DATABASE_URL'yi compose varsayılanıyla verir.
- Disk payı sınırlı: paralel worktree'lerde `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`; ortak `CARGO_TARGET_DIR` KULLANILMAZ (cargo çalışma alanı crate'lerini göreli yolla adlandırır, worktree'ler birbirinin test ikilisini ezer). Biten worktree'nin target'ı silinir.
- Bu ortamda imaj derlemesi: `make up BUILD_CA_FILE=/root/.ccr/ca-bundle.crt` (TLS vekili).
- CI: GitHub Actions, 6 iş akışı; iOS GitHub macOS makinelerinde. `altyapi.yml` actionlint, kustomize ve kind duman testini de koşar; kind ve compose uçtan uca denetimi CI'a bırakılır, yerelde ilgili crate testi yeterli. Gönderimden sonra CI sonucu `actions_list` ile denetlenir.
- DB rolleri: `app_owner` (sahip, NOLOGIN), `app_api`, `app_worker`; parolayı `migrate` verir (`APP_*_PASSWORD`).

## Engeller

- Vergi oranı tohumu yok; satırlar `unverified` başlar, resmi kaynakla doğrulanınca yüklenir (T2.8).
- T2.22: parti yalnız `approved` sürümden açılır (Evde de); taslaktan parti başlatılırsa sürüm önce yayınlanmalı.
- T2.10 kararları (itiraz gelmedikçe geçerli): bilinmeyen yol ve yanlış yöntem 404 (ADR kod listesinde 405 yok); idempotency yalnız 2xx saklar; hız sınırı IP 300, kullanıcı 600, çalışma alanı 1200 istek/dk; `/v1/meta/fields` `max-age=3600`; istek kimliği UUIDv7. Her yeni uç `api/src/openapi.rs` `OPERATIONS` tablosuna satır ekler. `/internal` dinleyicisi ve SSE sonraki işlerde.
- T2.30: `subscriptions`/`entitlements` yalnız `app_worker` yazar; destek elle düzeltmesi için yol kararlaştırılacak.
- T2.17 alan sınırları ürün önerisi (`verify: true`, doğrulanacak); katalog kesin birim tanımlarını, motor BeerSmith'in yuvarlak sabitlerini kullanıyor (~1e-7 fark), hizalama kararı açık.
- T2.28 web ödemesi O6 kararını bekler (App Store kısmı engelsiz). Açık kalanlar ADR-0002'de (Terraform hukuki/maliyet kararları, bulut hesabı O4, AppIcon).

## Son kararlar

- 11.11.2026: tam geliştirme; görüşme ve altın test Aşama 5 kapısı.
- ADR-0001 depo düzeni; ADR-0002 Aşama 0 kapanışı ve sapmalar.
- F1: formüller Fable ile bağımsız doğrulandı; 22 bulgu ve 14 karar işlendi. Kayan nokta ilkesi: gravite eşikleri tam sayı birimde (`round(SG*100000)`), sürekli büyüklükte 1e-9 (formuller/00-OKU-BENI.md).
- Her aşamaya yeni oturum.
- ADR-0006: Aşama 1 kapandı (13.11.2026); T1.17 motor eksikleri.
- ADR-0007: veri modeli. RLS + bileşik yabancı anahtarla çalışma alanı ayrımı, BYPASSRLS'siz uygulama rolleri, metrik birim ekli sütunlar, değişmez tarif sürümü, aylık bölümlü ölçüm, çalışma alanı başına özet zincirli denetim kaydı, EditSession Redis'te, bölge başına ayrı kurulum.
- ADR-0008: API kuralları. Çalışma alanı yolda, RFC 9457 hata, imleçli sayfalama, Merge Patch, ETag/If-Match (optimistic, field_lww, append), Idempotency-Key, içeriksiz SSE olayı, `/internal` ayrı port, oasdiff ile geriye uyumsuzluk denetimi.
- ADR-0009: kimlik, oturum, yetki. Apple/Google belirteci JWKS + tek kullanımlık nonce; opak oturum (özet PG'de, Redis önbellek); web çerez + CSRF, iOS Bearer; passkey ikinci adım (yöneticide zorunlu); ortak cihazda yalnız PIN (Face ID kişiyi ayırmaz; plan ve konsept güncellendi); izin listesi kodda, `api::authz` tek yer; Istio ambient mTLS; sırlar ortamdan veya `*_FILE`.
- ADR-0003: motor paritesi; bölüm başına modül; motor fonksiyonları spesifikasyon birimini alır; para `rust_decimal`.

## Aşama 2 (sürüyor)

T2.0 iş listesi · T2.1 veri modeli (ADR-0007) · T2.2 API kuralları (ADR-0008) · T2.3 kimlik ve yetki (ADR-0009) · T2.4 `db` crate, sqlx 0.9, göç servisi ve Job'ı · T2.17 alan, birim ve metin kataloğu (`sozlesme/katalog`, `make gen`, `common::catalog`) · T2.5 çekirdek göçler (23 tablo, FORCE RLS, üç rol, girişten önce SECURITY DEFINER işlevleri, RLS taraması) · T2.6 Ev MVP alan göçleri (0007–0013, `secure_workspace_table`, bölümlü ölçüm, değişmez sürüm, kapsam tetikleyicisi) · T2.10 API çekirdeği (bağlam ve `SET LOCAL`, RFC 9457, idempotency, ETag/If-Match, imleç, Redis hız sınırı fail-open, OTel, OpenAPI ve oasdiff, `/v1/meta/fields`). T2.7 İşletme ve vergi tabloları (0014–0015). 310 test geçiyor. Sıra: kararlar → veri ve F2.1 → API çekirdeği → alan uçları → destek → küme ve uçtan uca → F2.2 → kapı. Veri modeli İşletme tablolarını da taşır; İşletme uçları Aşama 6'da.

## Aşama 1 (kapandı)

T1.0 ADR-0003 · T1.1–T1.10 portlar · T1.14 referanssız hesaplar · T1.12 Fable incelemesi (5 rapor) · T1.15 bulgu düzeltmeleri · T1.16 ortak model, hata tipi, adlar (ADR-0004) · T1.11 ToolService, RecipeService, T1.11 bitti: 8 gRPC servisi (Tool, Recipe, Mash, Water, Yeast, Fermentation, Brewery, Engine), vektörlü uç testleri ve TCP üzerinden uçtan uca test; çalışma alanında 203 test geçiyor. T1.17 motor eksikleri · T1.13 kapı (ADR-0006). 10 269 parite vakası, hepsi eşit.

## Aşama 0 (kapandı)

T0.1 depo düzeni · T0.2 Rust iskeleti · T0.3 Docker Compose · T0.4 API sözleşmesi · T0.5 web · T0.6 iOS · T0.7 Kubernetes ve kind · T0.8 Terraform · T0.9 CI · T0.10 modül belgeleri · T0.11 kapı · T0.12 koyu yüksek karşıtlık · F1, F1.1, F1.2 formül doğrulaması
