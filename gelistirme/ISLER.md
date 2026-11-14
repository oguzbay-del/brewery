# İş listesi

Geliştirmenin tek takip yeri. Durum değişince bu dosya ve `DURUM.md` aynı commit'te güncellenir.

**Durum:** `bekliyor` · `sürüyor` · `bitti (tarih)` · `engelli (neden)`. İşin commit'i mesajı iş numarasıyla başlayan commit'tir (`git log --grep '^T0.2:'`).
**Kim:** `Claude` · `Oğuz` (ürün sahibi)
**Model:** işi kimin yaptığı. Opus işi ana oturumda yapılır; diğerleri alt ajana (Agent aracı) verilir.

## Model dağılımı

| Model | Ne zaman | Örnek |
|---|---|---|
| Opus (ana oturum) | Karar, tasarım, alt ajana iş tarifi yazma, sonucu inceleme, kapı kontrolü | Depo düzeni, veri modeli tasarımı, ADR, inceleme |
| Sonnet (alt ajan) | Tarifi net kod işi: yazma, test, derleme, hata düzeltme, belge | Rust iskeleti, Docker Compose, React ekranı, Terraform |
| Haiku (alt ajan) | Mekanik, tekrar eden, riski düşük | Şablondan dosya üretme, yeniden adlandırma, log tarama, sayım |
| Fable (alt ajan) | Yalnız kapılarda bağımsız doğrulama | Formül doğrulaması, veri modeli ve güvenlik incelemesi |

Alt ajan iş tarifi her zaman şunları içerir: okunacak dosyalar, yapılacak iş, bitti ölçütü, çalıştırılacak testler, "en fazla 20 satır rapor". Alt ajan commit atmaz; commit'i ana oturum inceledikten sonra atar.

## Aşama 0: Temel · KAPANDI (11.11.2026, ADR-0002)

Kapı: bütün sistem yerelde tek komutla ayağa kalkıyor, CI yeşil, kind üzerinde duman testi geçiyor.

| İş | Tanım | Bitti ölçütü | Bağlı | Kim | Model | Durum |
|---|---|---|---|---|---|---|
| T0.1 | Depo düzeni ve ADR-0001: klasörler (`sunucu/` Rust çalışma alanı, `web/`, `ios/`, `sozlesme/`, `altyapi/`), adlandırma, dal ve commit kuralları | `adr/0001-depo-duzeni.md` yazıldı, klasörler ve README'leri var | – | Claude | Opus | bitti (11.11.2026) |
| T0.2 | Rust çalışma alanı (`sunucu/`): `engine` (kütüphane), `engine-service` (gRPC), `api`, `worker`, `ingest`, `common`; her serviste sağlık ucu; `rust-toolchain.toml` | `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` geçiyor | T0.1 | Claude | Sonnet | bitti (11.11.2026) |
| T0.3 | Docker Compose: PostgreSQL, Redis, NATS, API, hesap motoru; sahte GitHub, e-posta ve APNs | `make up` ile hepsi ayağa kalkıyor, sağlık uçları 200 | T0.2 | Claude | Sonnet | bitti (11.11.2026) |
| T0.4 | API sözleşmesi iskeleti: OpenAPI koddan üretiliyor; Swift ve TypeScript istemci üretimi; gRPC proto | Üretim betiği çalışıyor, fark kontrolü CI'da | T0.2 | Claude | Sonnet | bitti (11.11.2026) |
| T0.5 | Web iskeleti: React + TypeScript (strict), tasarım token'ları `tokens.json`'dan CSS'e; Playwright duman testi | `npm test` ve Playwright geçiyor | T0.1 | Claude | Sonnet | bitti (11.11.2026) |
| T0.6 | iOS iskeleti: SwiftUI proje, token'lardan asset kataloğu üretimi, birim test hedefi | Proje dosyaları hazır; derleme macOS CI'da (T0.9) | T0.1 | Claude | Sonnet | bitti (11.11.2026; derleme macOS CI bekliyor) |
| T0.7 | Kubernetes tanımları (Helm veya Kustomize) ve kind duman testi betiği | `make kind-test` geçiyor | T0.3 | Claude | Sonnet | bitti (11.11.2026) |
| T0.8 | Terraform iskeleti (GKE Autopilot, Cloud SQL, Memorystore, Cloud Storage, Artifact Registry) | `terraform validate` geçiyor; `plan` için hesap gerekmez | T0.1 | Claude | Sonnet | bitti (11.11.2026) |
| T0.9 | CI: Rust, web, sözleşme farkı, Python test vektörleri, tutarlılık betiği, Compose duman testi; macOS iş akışı tanımı | İş akışı dosyaları yazıldı, yerelde `act` veya eşdeğer ile denendi | T0.2–T0.5 | Claude | Sonnet | bitti (11.11.2026; GitHub'da 6 iş akışı yeşil) |
| T0.10 | Modül README ve CLAUDE.md şablonları (ne yapar, neye dokunmaz, nasıl test edilir) | Her modülde var | T0.2, T0.5, T0.6 | Claude | Haiku | bitti (11.11.2026) |
| T0.11 | Aşama 0 kapı kontrolü ve inceleme | Kapı maddeleri sağlandı; sapmalar ADR'de | T0.3–T0.10 | Claude | Opus | bitti (11.11.2026) |
| T0.12 | Tasarım sistemi: koyu tema için yüksek karşıtlık değerleri (5 token); web ve iOS üretimi | Oranlar: metin 7:1, ayırıcı 3:1; web ve iOS üretim denetimleri geçiyor; tasarım sistemi yayını güncel | T0.5, T0.6 | Claude | Sonnet | bitti (11.11.2026) |

## Aşama 1: Hesap motoru · KAPANDI (13.11.2026, ADR-0006)

Kapı: Python referansıyla Rust motoru bütün test vektörlerinde eşit; Fable formül doğrulamasının bulguları kapandı.
Yöntem: ADR-0003. Bölüm işleri (T1.1–T1.10) birbirine dokunmaz, paralel yürür.

| İş | Tanım | Bitti ölçütü | Bağlı | Kim | Model | Durum |
|---|---|---|---|---|---|---|
| F1 | Formüllerin bağımsız doğrulaması: 03 su ve pH, 09 vergi yöntemleri, 10, 11 | Rapor `gelistirme/raporlar/F1-formul-dogrulama.md`; bulgular formüllere işlendi | – | Claude | Fable | bitti (11.11.2026; rapor: 22 bulgu, 2 kritik) |
| F1.1 | F1 bulgularını formüllere ve test vektörlerine işle; yargı gerektirenleri listele | Kritik ve önemli bulgular kapandı veya karar listesinde; `formuller/testler/*.py` geçiyor | F1 | Claude | Sonnet (işleme), Opus (inceleme) | bitti (11.11.2026; 14 karar maddesi F1.2) |
| F1.2 | F1 karar listesindeki 14 maddeyi incele, öneriyle ürün sahibine sun, kararları formüllere işle | `gelistirme/raporlar/F1-kararlar.md` her maddede karar var; formüller ve testler güncel | F1.1 | Claude | Opus (inceleme), Sonnet (işleme) | bitti (11.11.2026) |
| T1.0 | Aşama 1 iş listesi ve parite yöntemi (ADR-0003); parite altyapısı (`parite/ortak.py`, `tests/ortak`, `units`, modül iskeletleri, CI denetimi) | Bu tabloda T1.x satırları; `make vektor` ve `parite_00` geçiyor | T0.11 | Claude | Opus | bitti (11.11.2026) |
| T1.1 | 01 yoğunluk, alkol, renk → `density` | `parite/NN_*.py` vektörleri, `tests/parite_NN.rs` geçiyor; fmt, clippy temiz | T1.0 | Claude | Sonnet | bitti (11.11.2026) |
| T1.2 | 02 hop ve IBU → `hops` | `parite/NN_*.py` vektörleri, `tests/parite_NN.rs` geçiyor; fmt, clippy temiz | T1.0 | Claude | Sonnet | bitti (11.11.2026) |
| T1.3 | 03 su ve pH → `water` (NNLS dahil) | `parite/NN_*.py` vektörleri, `tests/parite_NN.rs` geçiyor; fmt, clippy temiz; NNLS saf Python referansıyla | T1.0 | Claude | Sonnet | bitti (11.11.2026) |
| T1.4 | 04 maya, mead, karbonasyon → `yeast` | `parite/NN_*.py` vektörleri, `tests/parite_NN.rs` geçiyor; fmt, clippy temiz | T1.0 | Claude | Sonnet | bitti (11.11.2026) |
| T1.5 | 05 mash ve hacimler → `mash` | `parite/NN_*.py` vektörleri, `tests/parite_NN.rs` geçiyor; fmt, clippy temiz | T1.0 | Claude | Sonnet | bitti (11.11.2026) |
| T1.6 | 07 Ev MVP ek hesaplar → `home` | `parite/NN_*.py` vektörleri, `tests/parite_NN.rs` geçiyor; fmt, clippy temiz | T1.0 | Claude | Sonnet | bitti (11.11.2026) |
| T1.7 | 08 İşletme çekirdek → `brewery` | `parite/NN_*.py` vektörleri, `tests/parite_NN.rs` geçiyor; fmt, clippy temiz | T1.0 | Claude | Sonnet | bitti (11.11.2026) |
| T1.8 | 09 İşletme 2: etiket enerjisi, vergi (ondalık), spunding → `brewery_tax` | `parite/NN_*.py` vektörleri, `tests/parite_NN.rs` geçiyor; fmt, clippy temiz; para `rust_decimal` | T1.0 | Claude | Sonnet | bitti (11.11.2026) |
| T1.9 | 10 elle giriş, ekstrakt, no-chill → `manual` | `parite/NN_*.py` vektörleri, `tests/parite_NN.rs` geçiyor; fmt, clippy temiz | T1.6 | Claude | Sonnet | bitti (11.11.2026) |
| T1.10 | 11 tetikleyici, fıçı, cask → `trigger` | `parite/NN_*.py` vektörleri, `tests/parite_NN.rs` geçiyor; fmt, clippy temiz | T1.4, T1.6 | Claude | Sonnet | bitti (11.11.2026) |
| T1.14 | Python referansı olmayan spesifikasyon hesapları: önce referans betiğe fonksiyon ve vektör, sonra Rust (bilinen: 02 §4–5, §7–9 tarif IBU, AAU, acılık ayarı, hop oranları; 05 §1, §3, §5.3–5.9 tahıl hacmi, emilim, BIAB, kaynatma sonrası, soğuma, buharlaşma, toplam su; 03 referanssız Rust korumaları, fosforik RA, SO₄/Cl sınıfları; 11 §1.1–1.2 ilerleme yüzdesi ve varsayılan Δ, §1.5, §3.1, §3.4 varsayılanları; 01, 04, 07, 08, 09, 10 açık yok) | Spesifikasyondaki her hesabın referansı ve parite vakası var | T1.1–T1.10 | Claude | Sonnet (yazım), Opus (inceleme) | bitti (11.11.2026; 01–05, 07–11 kapatıldı; UI metni, veri tabloları ve API birim dönüşümü dışarıda) |
| T1.15 | T1.12 bulgularını modül içinde düzelt (imza değiştirmeden): kritik ve önemli bulgular, zayıf parite vakaları; F1 gibi karar gerektirenler listeye | Her raporda kritik ve önemli bulgu kapandı ya da T1.16'ya/karar listesine taşındı; testler geçiyor | T1.12 | Claude | Sonnet (düzeltme), Opus (inceleme) | bitti (12.11.2026; ertelenenler `raporlar/T1.16-ertelenenler.md`) |
| T1.16 | Ortak tarif girdi modeli (`engine::recipe`), motor hata tipi ve girdi denetimi, birim adlandırması, yinelenen yardımcıların tekleşmesi (ADR-0004) | ADR-0004; 01/02 tek model; yinelenen fonksiyon yok; Python'un istisna attığı yerde Rust `Err`; testler geçiyor | T1.15 | Claude | Opus (ADR), Sonnet (uygulama) | bitti (12.11.2026; A hata ve kodlar, B ortak model, C adlar ve imzalar) |
| T1.11 | engine-service gRPC uçları: proto tasarımı (tarif özeti, su, mash hacimleri, karbonasyon, maya) ve uygulama; sözleşme üretimi | Proto ve OpenAPI fark kontrolü temiz; gRPC uç testleri geçiyor | T1.16 | Claude | Opus (proto), Sonnet (uygulama) | bitti (13.11.2026; 8 servis, ADR-0005; motor eksikleri `raporlar/T1.11-motor-eksikleri.md`) |
| T1.12 | Bağımsız inceleme: Rust ile spesifikasyon arasında sapma, kapsanmayan fonksiyon | Rapor `gelistirme/raporlar/T1.12-motor-inceleme.md`; bulgular kapandı | T1.1–T1.10, T1.14 | Claude | Fable | bitti (12.11.2026; 5 rapor) |
| T1.17 | T1.11 motor eksikleri ve açık T1.12 bulguları: 03 tipli grist ve çok asitli pH; 04 maya alan adı; 07 C10 "bitti" durumu, Tilt düzeltmesi, 11 §1.3 elle gün çözünürlüklü tetik; 08 §4.3 süzme verimi ve lab oranı; 09 LPA ve uygulanan oran çıktısı, spunding sabiti (`raporlar/T1.11-motor-eksikleri.md`) | Spesifikasyon → referans → vektör → Rust → servis; parite ve uç testleri geçiyor | T1.11 | Claude | Sonnet (yazım), Opus (inceleme) | bitti (13.11.2026) |
| T1.13 | Aşama 1 kapı kontrolü | Kapı maddeleri sağlandı; sapmalar ADR'de | T1.11, T1.12, T1.17 | Claude | Opus | bitti (13.11.2026; ADR-0006) |

## Aşama 2: Çekirdek API ve veri · SÜRÜYOR

Kapı: uçtan uca testler compose'da ve kind'de yeşil; Fable veri modeli (F2.1) ve güvenlik (F2.2) incelemelerinin kritik ve önemli bulguları kapandı; birleştirme kapısı (fmt, clippy, test, sqlx denetimi, sözleşme farkı, `araclar/tutarlilik.py`) yeşil.
Kapsam: veri modeli İşletme tablolarını da taşır (plan › Veri modeli); uçlar Ev MVP ekranlarının ihtiyacı kadar, İşletme uçları Aşama 6'da. Sıra: kararlar (T2.1–T2.3) → veri (T2.4–T2.9) → API çekirdeği (T2.10–T2.18) → alan uçları (T2.19–T2.28) → destek (T2.29–T2.31) → küme ve uçtan uca (T2.32–T2.33) → güvenlik incelemesi ve kapı. Aynı bağımlılığı olan işler paralel yürür; her alan işi kendi göç ekini, uç testini ve OpenAPI farkını getirir.

| İş | Tanım | Bitti ölçütü | Bağlı | Kim | Model | Durum |
|---|---|---|---|---|---|---|
| T2.0 | Aşama 2 iş listesi | Bu tabloda T2.x satırları; DURUM.md güncel | T1.13 | Claude | Opus | bitti (13.11.2026) |
| T2.1 | Veri modeli (ADR-0007): plan › Veri modeli'ndeki her varlığın tablo tasarımı (Ev MVP ve İşletme birlikte); ortak sütunlar (UUIDv7, oluşturma, güncelleme, silinme zamanı, sürüm numarası); iç birim SI ve sütun adında birim; sabit sütun ile JSONB sınırı; değişmez tarif sürümü ve IngredientUse anlık kopyası; çalışma alanı ayrımı (RLS, `app.workspace_id`); ortak ve çalışma alanına özel kütüphane; Measurement bölümlemesi; outbox ve idempotency tabloları; AuditLog özet zinciri; silme, saklama ve anonimleştirme; veri yeri (bölge) | ADR-0007 ve ER diyagramı (mermaid); her plan varlığının tablo karşılığı ya da gerekçeli dışarıda bırakılışı | T2.0 | Claude | Opus | bitti (13.11.2026; ADR-0007) |
| T2.2 | API kuralları (ADR-0008): kaynak ve yol adları; hata zarfı ve kod listesi; cursor sayfalama; idempotency anahtarı; ETag ve If-Match ile iyimser kilit, çakışma gövdesi (fark); alan tanımlarının (min, max, birim, zorunlu) veri olarak sunulması; API'de SI birim; yetkiye göre alan gizleme (maliyet); SSE olay biçimi; `/v1` ve `/internal` ayrımı; en düşük uygulama sürümü; OpenAPI geriye uyumsuzluk denetimi | ADR-0008 | T2.0 | Claude | Opus | bitti (13.11.2026) |
| T2.3 | Kimlik, oturum ve yetki (ADR-0009): Apple ve Google kimlik belirteci doğrulaması (JWKS, iss, aud, nonce); e-posta bağlantısı; opak oturum belirteci (özeti PostgreSQL'de, Redis önbelleği, süreler); web çerez ve CSRF, mobilde Bearer; passkey (WebAuthn) ve yönetici ikinci adımı; hesap bağlama; davet; ortak cihaz PIN'i; izin listesi, rol ve kapsam modeli; Ev modunda paylaşım; servisler arası mTLS yöntemi; sırlar (yerel ve küme) | ADR-0009; kısa tehdit listesi (STRIDE) | T2.1 | Claude | Opus | bitti (13.11.2026) |
| T2.4 | `db` crate ve göç altyapısı: sqlx (PostgreSQL, çevrimdışı `.sqlx` sorgu önbelleği), `sunucu/migrations/`, `make migrate`, `make sqlx-prepare`, test başına geçici veritabanı, compose'da göç adımı, Kubernetes'te göç Job'ı, CI'da sqlx denetimi | Boş göçle `make up` ve `cargo test -p db` geçiyor; CI'da `cargo sqlx prepare --check` | T2.1 | Claude | Sonnet | bitti (13.11.2026) |
| T2.5 | Çekirdek göçler: Workspace, User, UserIdentity, passkey, oturum, Role, Permission, RoleAssignment, Invitation, DeviceSession, AuditLog, outbox, idempotency; RLS politikaları; tablo sahibi olmayan, RLS'yi aşamayan uygulama rolü | Göçler temiz; RLS testi: A çalışma alanının bağlamında B'nin satırı okunmuyor ve yazılmıyor; bağlam yoksa satır görünmüyor | T2.3, T2.4 | Claude | Sonnet | bitti (13.11.2026) |
| T2.6 | Ev MVP alan göçleri: Ingredient, StyleGuide, Style, StyleMapping, Equipment, Profile, Recipe, RecipeVersion, IngredientUse, EditSession, Batch, BatchStep, BatchIngredientActual, Deviation, Measurement (bölümlü), StageEvent, StockLot, StockMovement, PackageType, Notification, NotificationEvent, Reminder, Device, AlertRule, BrewDayPackage, CalcResult, Attachment, ScannedDocument, ImportSource, Subscription, Entitlement, SupportRequest, SupportMessage, SupportGrant, SupportAction | Her tabloda ortak sütunlar ve RLS; RLS testi bütün tabloları tarar, politikası olmayan tablo testi düşürür | T2.5 | Claude | Sonnet | bitti (13.11.2026) |
| T2.7 | İşletme göçleri: Vessel, VesselOccupancy, Transfer, PackagingRun, QualitySpec, LabResult, Release, CleaningRecord, YeastGeneration; vergi tabloları (ülke, yürürlük tarihi, bant, kademe, indirim) | Aynı RLS taraması geçiyor; uç yok (Aşama 6) | T2.6 | Claude | Sonnet | bitti (13.11.2026) |
| T2.8 | 09 vergi yöntem zinciri (bant + kademe + indirim) ve yıl başından hacim (`ytd_hl`) taşınması; tablo verisi motora girdi olarak (ADR-0006 açığı) | Spesifikasyon → referans → vektör → Rust → servis; parite ve uç testleri geçiyor | T2.7 | Claude | Sonnet (yazım), Opus (inceleme) | bitti (14.11.2026) |
| F2.1 | Bağımsız veri modeli incelemesi: ADR-0007 ve göçler; plan › Veri modeli, Eşzamanlı düzenleme, Güvenlik ile karşılaştırma; kısıt, indeks, RLS boşluğu, silme ve anonimleştirme | Rapor `gelistirme/raporlar/F2.1-veri-modeli.md` | T2.7 | Claude | Fable | bitti (14.11.2026) |
| T2.9 | F2.1 bulgularının düzeltmesi | Kritik ve önemli bulgu kapandı ya da karar listesinde; testler geçiyor | F2.1 | Claude | Sonnet (düzeltme), Opus (inceleme) | bekliyor |
| T2.10 | API çekirdeği: uygulama durumu, PostgreSQL havuzu, Redis istemcisi; istek bağlamı (kullanıcı, çalışma alanı, izinler) ve işlem başına `SET LOCAL`; hata zarfı; idempotency ara katmanı; If-Match ve ETag; cursor sayfalama; hız sınırı (Redis); istek kimliği ve OpenTelemetry izi; OpenAPI üretimi | ADR-0008'in her kuralına test; Redis kapatılınca API çalışmaya devam ediyor (test) | T2.2, T2.5 | Claude | Sonnet | bitti (13.11.2026) |
| T2.11 | Giriş: Apple ve Google kimlik belirteci doğrulaması (JWKS önbelleği); e-posta bağlantısı (mailpit); ilk girişte kullanıcı ve çalışma alanı (mod, bölge); oturum, çıkış, her yerden çıkış, oturum listesi; hesap bağlama ve "aynı e-posta başka hesapta"; compose'a sahte OIDC sağlayıcısı | Uçtan uca: sahte Apple ve Google belirteciyle giriş; yanlış aud, iss, süre reddi; e-posta bağlantısı tek kullanımlık ve süreli | T2.3, T2.10 | Claude | Sonnet | bitti (14.11.2026) |
| T2.12 | Passkey ve ortak cihaz: WebAuthn kaydı ve doğrulaması; yönetici işlemlerinde ikinci adım zorunluluğu; ortak cihaz modu (yönetici açar, kişiye özel PIN, işlemsizlik kilidi), DeviceSession | Uç testleri; PIN deneme sınırı; ikinci adımsız yönetici işlemi reddi | T2.11 | Claude | Sonnet | bekliyor |
| T2.13 | Yetki: izin listesi, hazır rollerin tohumu (plan › Yetki matrisi), rol düzenleme ve kopyalama, RoleAssignment ve kapsam, son yönetici koruması, davet (tek kullanımlık, 7 gün), Ev modunda tarif ve parti paylaşımı (düzenleyici, izleyici), maliyet alanlarının yanıttan çıkarılması | Matrisin her hücresine test (rol × izin); yetkisiz istek 403, başka çalışma alanının kaydı 404 | T2.11 | Claude | Sonnet | bekliyor |
| T2.14 | Denetim kaydı: yalnız eklenebilir tablo (uygulama rolünde UPDATE ve DELETE yok), her satırda öncekinin özeti, zinciri doğrulayan komut; rol, yetki ve kapanmış parti düzeltmeleri yazılır | Satır silinince veya değişince doğrulama hata veriyor (test) | T2.10 | Claude | Sonnet | bitti (13.11.2026) |
| T2.15 | Olaylar: outbox → NATS JetStream aktarıcısı (worker); tüketici çerçevesi (aynı olay iki kez işlenmez, yeniden deneme, ölü mektup); SSE ucu (çalışma alanı ve kayıt kanalları, podlar arası) | Uçtan uca: kayıt yazılınca SSE istemcisi olayı alıyor; NATS yeniden başlayınca olay kaybolmuyor | T2.10 | Claude | Sonnet | bitti (14.11.2026) |
| T2.16 | Motor bağlantısı: API'den engine-service gRPC istemcisi; motor sürüm damgası; girdi özetiyle Redis önbelleği; CalcResult kaydı; motor hata kodlarının API hata zarfına eşlenmesi; "Nasıl hesaplandı" ara değerleri | Aynı girdi ikinci kez motora gitmiyor (test); motor kapalıyken anlamlı hata | T2.10 | Claude | Sonnet | bitti (13.11.2026) |
| T2.17 | Alan tanımları, birim ve metin kataloğu: alan sınırlarının tek kaynağı ve API'den sunulması; birim dönüşüm katsayıları ve TR/EN metin kataloğu tek veri dosyasında, TypeScript ve Swift'e üretim (`make gen`) | Üretilen dosyalar fark denetiminde; sunucu doğrulaması aynı tanımı kullanıyor | T2.2 | Claude | Sonnet | bitti (13.11.2026) |
| T2.18 | Nesne deposu: tek arayüz (S3, GCS, Azure Blob), compose'a S3 uyumlu yerel servis, kısa süreli imzalı yükleme ve indirme bağlantısı, boyut ve tür sınırı; Attachment ve ScannedDocument uçları | Uçtan uca yükleme ve indirme; süresi geçmiş bağlantı reddi | T2.10 | Claude | Sonnet | bitti (14.11.2026) |
| T2.19 | Malzeme kütüphanesi ve stil kılavuzları: ortak ve çalışma alanına özel malzeme, kendi malzemesini ekleme, malzeme detayı; stil kılavuzu, stil ve kılavuzlar arası eşleşme uçları; örnek tohum verisi ayrı komutla ("örnek" işaretli; gerçek kütüphane verisinin kaynağı ayrı karar) | Uç testleri geçiyor | T2.9, T2.13 | Claude | Sonnet | bekliyor |
| T2.20 | Ekipman, profiller ve ayarlar: Equipment (ekipman sihirbazı), mash, kaynatma, fermantasyon, karbonasyon ve su profilleri, su analiz raporu, kaplar; çalışma alanı ve kullanıcı ayarları (birim, hesaplama seçenekleri); alan bazında son yazan kazanır ve değişiklik geçmişi; kalibrasyon önerisi (motor) | Uç testleri; geçmişten geri alma testi | T2.9, T2.13, T2.16 | Claude | Sonnet | bekliyor |
| T2.21 | Tarif: Recipe, taslak ve yayınlanmış değişmez sürüm; İşletme onay durumu alanı (akış uçları Aşama 6); IngredientUse anlık kopyası; hesap (T2.16); ölçekleme; tarif ve sürüm karşılaştırma; EditSession yumuşak kilidi (SSE); klasör, etiket, favori; mead, cider, seltzer; mead bal yüzdesi ve `est_cells_ml` (ADR-0006 açıkları) | Uçtan uca: tarif → hesap → yayınla → eski sürüm değişmiyor; eski sürümle kayıt 412 ve fark | T2.19, T2.20 | Claude | Sonnet | bekliyor |
| T2.22 | Parti ve demleme paketi: tariften parti başlatma; BatchStep; gerçek miktar ve sapma; ölçümler (istemci UUID'siyle tekrar güvenli); aşama geçişleri; parti özeti (plan–gerçekleşen); takvim ve geriye planlama; demleme paketi (sunucuda hesaplanan adım, sayaç, hedef; imzalı ve süreli); paket kimliğiyle ölçüm kabulü | Uçtan uca: aynı ölçüm iki kez gelince tek kayıt; başka partinin paketiyle ölçüm reddi; süresi dolmuş paket reddi | T2.21 | Claude | Sonnet | bekliyor |
| T2.23 | Stok ve alışveriş listesi: StockLot (evde tek lot), hareket defteri, toplamların hareketlerden hesabı, eksi stok uyarısı, mal kabul ve sayım, partide düşüm, tariften alışveriş listesi | Uç testleri; eşzamanlı iki hareketten doğru toplam | T2.22 | Claude | Sonnet | bekliyor |
| T2.24 | Bildirimler: Notification, NotificationEvent, Reminder; üç öncelik, sessiz saatler, günlük özet; push (sahte APNs) ve e-posta (mailpit) worker'da; bildirim kutusu ve eylemler (onayla, ertele, yoksay); uyarı kuralları (AlertRule) ölçüm olaylarında değerlendirilir | Uçtan uca: ölçüm → kural → bildirim → sahte APNs'e istek | T2.15, T2.22 | Claude | Sonnet | bekliyor |
| T2.25 | Cihaz verisi alımı (ingest): cihaz kaydı ve eşleştirme; cihaz başına yenilenebilir anahtarlı adres; iSpindel ve RAPT biçimleri; aralık dışı değer reddi; NATS'a yazma, worker'da Measurement'a işleme | Uçtan uca: iSpindel isteği → ölçüm → SSE; yenilenmiş anahtardan sonra eski anahtar reddi | T2.15, T2.22 | Claude | Sonnet | bekliyor |
| T2.26 | İçe ve dışa aktarma: BeerXML ve BeerJSON içe aktarma (güvenli XML ayrıştırıcı, boyut sınırı), eşleştirme kararları (ImportSource); dışa aktarma (BeerJSON, BeerXML, kaçışlı CSV, JSON, ZIP) worker işi olarak; `.bsmx` örnek dosya bulunursa | Örnek dosyalarla gidiş-dönüş testi; XML bombası ve dış varlık reddi | T2.18, T2.21 | Claude | Sonnet | bekliyor |
| T2.27 | Global arama: PostgreSQL tam metin araması (tarif, parti, malzeme, stok), çalışma alanıyla sınırlı | Uç testi; başka çalışma alanından sonuç yok | T2.23 | Claude | Sonnet | bekliyor |
| T2.28 | Abonelik: Subscription, Entitlement; 1 ay tam deneme, sonrasında ücretli özellikler salt okunur, dışa aktarma hep açık; App Store Server Notifications v2 (JWS doğrulaması) ingest'te; web ödeme sağlayıcısı arayüzü (sağlayıcı O6 kararıyla) | Uçtan uca: yenileme, iade, deneme bitişi; aynı bildirim iki kez işlenmiyor | T2.13, T2.15 | Claude | Sonnet | bekliyor |
| T2.29 | Destek talepleri: SupportRequest, SupportMessage, talep durumları, takma kimlik, ekler nesne deposunda; GitHub App ile eşitleme (outbox → GitHub API, sınırda yeniden deneme); GitHub webhook'u ingest'te (imza, teslim tekrarı); `/yanit` kuralı ve destek ekibi listesi; kendi yorumunu yok sayma; düzenleme ve silme yansıması; sahte GitHub (wiremock) | Uçtan uca: talep → Issue; `/yanit` → bildirim; iç not müşteriye gitmiyor; GitHub kapalıyken talep açılıyor, sonra eşitleniyor | T2.15, T2.18, T2.24 | Claude | Sonnet | bekliyor |
| T2.30 | Destek paneli uçları (`/internal`): hesap arama ve özet, abonelik yeniden doğrulama ve gerekçeli düzeltme, oturum kapatma, cihaz anahtarı yenileme, silme talepleri; SupportGrant (72 saat, gerekçe, talep numarası), iki onaylı acil erişim; SupportAction denetim kaydı ve müşteride "Destek erişim geçmişi" | İzinsiz içerik isteği 403; izin süresi bitince erişim kapanıyor; her işlem denetim kaydında | T2.13, T2.14, T2.29 | Claude | Sonnet | bekliyor |
| T2.31 | Hesap silme ve veri dışa aktarma talebi: uygulama içinden silme, Apple belirteci iptali, saklama süresi ve geri getirme, anonimleştirme, GitHub Issue silme veya anonimleştirme, tam dışa aktarma (ZIP) | Uçtan uca: silinen hesabın verisi yok, denetim kaydı anonim, zincir doğrulaması geçiyor | T2.26, T2.30 | Claude | Sonnet | bekliyor |
| T2.32 | Küme: kind'de PostgreSQL, Redis, nesne deposu ve sahte servisler (yerel overlay); göç Job'ı; servisler arası mTLS (ADR-0009); ağ politikaları (motora yalnız API, `/internal` yalnız OIDC ters vekilinden); oauth2-proxy; sırlar; Terraform'a Cloud SQL, Memorystore, Cloud Storage modülleri | `make kind-test` yeni denetimlerle geçiyor (genel girişten `/internal` 404); `terraform validate` temiz | T2.11, T2.15, T2.30 | Claude | Sonnet | bekliyor |
| T2.36 | Görsel mimari akış şeması (üst seviye): istemciler (iPhone/iPad, web), kenar, uygulama servisleri (api, worker, ingest), hesap motoru, veri (PostgreSQL, Redis, NATS, nesne deposu) ve bütün dış bağlantılar kutu ve tek ok olarak (Apple giriş, Google giriş, App Store, APNs, e-posta, GitHub, ölçüm cihazları iSpindel/RAPT, web ödeme sağlayıcısı, gözlem/OTel, bulut); bağlantı başına yalnız ad ve yön, ayrıntı yok | `belgeler/mimari-akis.md` içinde mermaid şema ve yayımlanmış görsel sayfa (Artifact); plan › Platformlar ve mimari ile tutarlı; her dış bağlantı bir kez görünüyor | T2.32 | Claude | Sonnet | bekliyor |
| T2.33 | Uçtan uca takım: compose üzerinde senaryolar (Ev kullanıcısının tam akışı: giriş → kurulum → tarif → parti → paket → ölçüm → bildirim → destek talebi; iki çalışma alanı ayrımı); `make e2e` ve CI iş akışı; kind'de kısa takım | `make e2e` yerelde ve CI'da yeşil | T2.21–T2.32 | Claude | Sonnet | bekliyor |
| F2.2 | Bağımsız güvenlik incelemesi: kimlik, oturum, yetki, çalışma alanı ayrımı, demleme paketi imzası, webhook imzaları, destek paneli erişimi, dosya yükleme, içe aktarma | Rapor `gelistirme/raporlar/F2.2-guvenlik.md` | T2.33 | Claude | Fable | bekliyor |
| T2.34 | F2.2 bulgularının düzeltmesi | Kritik ve önemli bulgu kapandı ya da karar listesinde; uçtan uca takım yeşil | F2.2 | Claude | Sonnet (düzeltme), Opus (inceleme) | bekliyor |
| T2.35 | Aşama 2 kapı kontrolü | Kapı maddeleri sağlandı; sapmalar ADR'de | T2.9, T2.34, T2.36 | Claude | Opus | bekliyor |

## Aşama 3–6

Aşamalar plan › Geliştirme düzeni ve plan › Yol haritasında. Her aşamanın iş listesi bir önceki aşama kapandığında yazılır (Opus).

| Aşama | İçerik | Kapı |
|---|---|---|
| 3 Web | Ev MVP ekranları | Playwright, erişilebilirlik |
| 4 iOS | SwiftUI, demleme paketi, Live Activity | macOS CI yeşil; cihaz testi (Oğuz) |
| 5 Ev MVP beta | Bulut kurulumu, beta | Görüşmeler ve altın test tamam; bağımsız güvenlik incelemesi |
| 6 İşletme | İşletme ekranları | Aşama aşama |

## Oğuz'un işleri (paralel)

| İş | Tanım | Gerektiği yer | Durum |
|---|---|---|---|
| O1 | Kullanıcı görüşmeleri (`plan/gorusme.md`) | Aşama 5 kapısı | bekliyor |
| O2 | BeerSmith ekranından altın test sayıları (`formuller/ALTIN-TEST.md`) | Aşama 5 kapısı | bekliyor |
| O3 | Apple Developer hesabı, macOS CI (GitHub Actions macOS veya Xcode Cloud) | T0.9, Aşama 4 | kısmen: macOS CI GitHub'ın macOS makinelerinde çalışıyor; Apple Developer hesabı ve cihaz testi bekliyor |
| O4 | Google Cloud organizasyonu ve faturalandırma | Aşama 5 | bekliyor |
| O5 | GitHub'da eski 7 dalı silme | – | bekliyor |
| O6 | Web ödeme sağlayıcısı seçimi (App Store dışındaki abonelik) | T2.28 | bekliyor |
