# Task list

This is the only place to track development. When a status changes, update this file and `DURUM.md` in the same commit. Write new and changed rows in ASD-STE100 (root `CLAUDE.md` › Yazım standardı). Do not rewrite the Turkish text of closed stages and done tasks.

**Status:** `todo` · `active` · `done (YYYY-MM-DD)` · `blocked (reason)`. The commit of a task is the commit with a message that starts with the task number (`git log --grep '^T0.2:'`).
**Owner:** `Claude` · `ChatGPT` · `Oğuz` (product owner). A task with a GitHub issue shows the issue number (`#193`). ChatGPT rules: `marketing-site/CHATGPT-SCOPE.md`.
**Model:** the model that does the task. The main session does Opus tasks. Give the other tasks to a subagent (Agent tool).

## Model distribution

| Model | When | Example |
|---|---|---|
| Opus (main session) | Decisions, design, task briefs for subagents, review of results, gate checks | Repository layout, data model design, ADR, review |
| Sonnet (subagent) | Code tasks with a clear brief: write, test, build, fix errors, documents | Rust skeleton, Docker Compose, React screen, Terraform |
| Haiku (subagent) | Mechanical, repetitive tasks with low risk | Files from a template, renames, log scans, counts |
| Fable (subagent) | Only independent checks at gates | Formula check, data model and security review |

A subagent brief always gives: the files to read, the task, the done criterion, the tests to run, and "report in at most 20 lines". Write the brief in STE. A subagent does not commit. The main session commits after it examines the result.

## Stage 0: Foundation · CLOSED (2026-11-11, ADR-0002)

Kapı: bütün sistem yerelde tek komutla ayağa kalkıyor, CI yeşil, kind üzerinde duman testi geçiyor.

| Task | Description | Done criterion | Depends on | Owner | Model | Status |
|---|---|---|---|---|---|---|
| T0.1 | Depo düzeni ve ADR-0001: klasörler (`sunucu/` Rust çalışma alanı, `web/`, `ios/`, `sozlesme/`, `altyapi/`), adlandırma, dal ve commit kuralları | `adr/0001-depo-duzeni.md` yazıldı, klasörler ve README'leri var | – | Claude | Opus | done (2026-11-11) |
| T0.2 | Rust çalışma alanı (`sunucu/`): `engine` (kütüphane), `engine-service` (gRPC), `api`, `worker`, `ingest`, `common`; her serviste sağlık ucu; `rust-toolchain.toml` | `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` geçiyor | T0.1 | Claude | Sonnet | done (2026-11-11) |
| T0.3 | Docker Compose: PostgreSQL, Redis, NATS, API, hesap motoru; sahte GitHub, e-posta ve APNs | `make up` ile hepsi ayağa kalkıyor, sağlık uçları 200 | T0.2 | Claude | Sonnet | done (2026-11-11) |
| T0.4 | API sözleşmesi iskeleti: OpenAPI koddan üretiliyor; Swift ve TypeScript istemci üretimi; gRPC proto | Üretim betiği çalışıyor, fark kontrolü CI'da | T0.2 | Claude | Sonnet | done (2026-11-11) |
| T0.5 | Web iskeleti: React + TypeScript (strict), tasarım token'ları `tokens.json`'dan CSS'e; Playwright duman testi | `npm test` ve Playwright geçiyor | T0.1 | Claude | Sonnet | done (2026-11-11) |
| T0.6 | iOS iskeleti: SwiftUI proje, token'lardan asset kataloğu üretimi, birim test hedefi | Proje dosyaları hazır; derleme macOS CI'da (T0.9) | T0.1 | Claude | Sonnet | done (2026-11-11; the build waits for macOS CI) |
| T0.7 | Kubernetes tanımları (Helm veya Kustomize) ve kind duman testi betiği | `make kind-test` geçiyor | T0.3 | Claude | Sonnet | done (2026-11-11) |
| T0.8 | Terraform iskeleti (GKE Autopilot, Cloud SQL, Memorystore, Cloud Storage, Artifact Registry) | `terraform validate` geçiyor; `plan` için hesap gerekmez | T0.1 | Claude | Sonnet | done (2026-11-11) |
| T0.9 | CI: Rust, web, sözleşme farkı, Python test vektörleri, tutarlılık betiği, Compose duman testi; macOS iş akışı tanımı | İş akışı dosyaları yazıldı, yerelde `act` veya eşdeğer ile denendi | T0.2–T0.5 | Claude | Sonnet | done (2026-11-11; GitHub'da 6 iş akışı yeşil) |
| T0.10 | Modül README ve CLAUDE.md şablonları (ne yapar, neye dokunmaz, nasıl test edilir) | Her modülde var | T0.2, T0.5, T0.6 | Claude | Haiku | done (2026-11-11) |
| T0.11 | Aşama 0 kapı kontrolü ve inceleme | Kapı maddeleri sağlandı; sapmalar ADR'de | T0.3–T0.10 | Claude | Opus | done (2026-11-11) |
| T0.12 | Tasarım sistemi: koyu tema için yüksek karşıtlık değerleri (5 token); web ve iOS üretimi | Oranlar: metin 7:1, ayırıcı 3:1; web ve iOS üretim denetimleri geçiyor; tasarım sistemi yayını güncel | T0.5, T0.6 | Claude | Sonnet | done (2026-11-11) |

## Stage 1: Calculation engine · CLOSED (2026-11-13, ADR-0006)

Kapı: Python referansıyla Rust motoru bütün test vektörlerinde eşit; Fable formül doğrulamasının bulguları kapandı.
Yöntem: ADR-0003. Bölüm işleri (T1.1–T1.10) birbirine dokunmaz, paralel yürür.

| Task | Description | Done criterion | Depends on | Owner | Model | Status |
|---|---|---|---|---|---|---|
| F1 | Formüllerin bağımsız doğrulaması: 03 su ve pH, 09 vergi yöntemleri, 10, 11 | Rapor `gelistirme/raporlar/F1-formul-dogrulama.md`; bulgular formüllere işlendi | – | Claude | Fable | done (2026-11-11; rapor: 22 bulgu, 2 kritik) |
| F1.1 | F1 bulgularını formüllere ve test vektörlerine işle; yargı gerektirenleri listele | Kritik ve önemli bulgular kapandı veya karar listesinde; `formuller/testler/*.py` geçiyor | F1 | Claude | Sonnet (işleme), Opus (inceleme) | done (2026-11-11; 14 karar maddesi F1.2) |
| F1.2 | F1 karar listesindeki 14 maddeyi incele, öneriyle ürün sahibine sun, kararları formüllere işle | `gelistirme/raporlar/F1-kararlar.md` her maddede karar var; formüller ve testler güncel | F1.1 | Claude | Opus (inceleme), Sonnet (işleme) | done (2026-11-11) |
| T1.0 | Aşama 1 iş listesi ve parite yöntemi (ADR-0003); parite altyapısı (`parite/ortak.py`, `tests/ortak`, `units`, modül iskeletleri, CI denetimi) | Bu tabloda T1.x satırları; `make vektor` ve `parite_00` geçiyor | T0.11 | Claude | Opus | done (2026-11-11) |
| T1.1 | 01 yoğunluk, alkol, renk → `density` | `parite/NN_*.py` vektörleri, `tests/parite_NN.rs` geçiyor; fmt, clippy temiz | T1.0 | Claude | Sonnet | done (2026-11-11) |
| T1.2 | 02 hop ve IBU → `hops` | `parite/NN_*.py` vektörleri, `tests/parite_NN.rs` geçiyor; fmt, clippy temiz | T1.0 | Claude | Sonnet | done (2026-11-11) |
| T1.3 | 03 su ve pH → `water` (NNLS dahil) | `parite/NN_*.py` vektörleri, `tests/parite_NN.rs` geçiyor; fmt, clippy temiz; NNLS saf Python referansıyla | T1.0 | Claude | Sonnet | done (2026-11-11) |
| T1.4 | 04 maya, mead, karbonasyon → `yeast` | `parite/NN_*.py` vektörleri, `tests/parite_NN.rs` geçiyor; fmt, clippy temiz | T1.0 | Claude | Sonnet | done (2026-11-11) |
| T1.5 | 05 mash ve hacimler → `mash` | `parite/NN_*.py` vektörleri, `tests/parite_NN.rs` geçiyor; fmt, clippy temiz | T1.0 | Claude | Sonnet | done (2026-11-11) |
| T1.6 | 07 Ev MVP ek hesaplar → `home` | `parite/NN_*.py` vektörleri, `tests/parite_NN.rs` geçiyor; fmt, clippy temiz | T1.0 | Claude | Sonnet | done (2026-11-11) |
| T1.7 | 08 İşletme çekirdek → `brewery` | `parite/NN_*.py` vektörleri, `tests/parite_NN.rs` geçiyor; fmt, clippy temiz | T1.0 | Claude | Sonnet | done (2026-11-11) |
| T1.8 | 09 İşletme 2: etiket enerjisi, vergi (ondalık), spunding → `brewery_tax` | `parite/NN_*.py` vektörleri, `tests/parite_NN.rs` geçiyor; fmt, clippy temiz; para `rust_decimal` | T1.0 | Claude | Sonnet | done (2026-11-11) |
| T1.9 | 10 elle giriş, ekstrakt, no-chill → `manual` | `parite/NN_*.py` vektörleri, `tests/parite_NN.rs` geçiyor; fmt, clippy temiz | T1.6 | Claude | Sonnet | done (2026-11-11) |
| T1.10 | 11 tetikleyici, fıçı, cask → `trigger` | `parite/NN_*.py` vektörleri, `tests/parite_NN.rs` geçiyor; fmt, clippy temiz | T1.4, T1.6 | Claude | Sonnet | done (2026-11-11) |
| T1.14 | Python referansı olmayan spesifikasyon hesapları: önce referans betiğe fonksiyon ve vektör, sonra Rust (bilinen: 02 §4–5, §7–9 tarif IBU, AAU, acılık ayarı, hop oranları; 05 §1, §3, §5.3–5.9 tahıl hacmi, emilim, BIAB, kaynatma sonrası, soğuma, buharlaşma, toplam su; 03 referanssız Rust korumaları, fosforik RA, SO₄/Cl sınıfları; 11 §1.1–1.2 ilerleme yüzdesi ve varsayılan Δ, §1.5, §3.1, §3.4 varsayılanları; 01, 04, 07, 08, 09, 10 açık yok) | Spesifikasyondaki her hesabın referansı ve parite vakası var | T1.1–T1.10 | Claude | Sonnet (yazım), Opus (inceleme) | done (2026-11-11; 01–05, 07–11 kapatıldı; UI metni, veri tabloları ve API birim dönüşümü dışarıda) |
| T1.15 | T1.12 bulgularını modül içinde düzelt (imza değiştirmeden): kritik ve önemli bulgular, zayıf parite vakaları; F1 gibi karar gerektirenler listeye | Her raporda kritik ve önemli bulgu kapandı ya da T1.16'ya/karar listesine taşındı; testler geçiyor | T1.12 | Claude | Sonnet (düzeltme), Opus (inceleme) | done (2026-11-12; ertelenenler `raporlar/T1.16-ertelenenler.md`) |
| T1.16 | Ortak tarif girdi modeli (`engine::recipe`), motor hata tipi ve girdi denetimi, birim adlandırması, yinelenen yardımcıların tekleşmesi (ADR-0004) | ADR-0004; 01/02 tek model; yinelenen fonksiyon yok; Python'un istisna attığı yerde Rust `Err`; testler geçiyor | T1.15 | Claude | Opus (ADR), Sonnet (uygulama) | done (2026-11-12; A hata ve kodlar, B ortak model, C adlar ve imzalar) |
| T1.11 | engine-service gRPC uçları: proto tasarımı (tarif özeti, su, mash hacimleri, karbonasyon, maya) ve uygulama; sözleşme üretimi | Proto ve OpenAPI fark kontrolü temiz; gRPC uç testleri geçiyor | T1.16 | Claude | Opus (proto), Sonnet (uygulama) | done (2026-11-13; 8 servis, ADR-0005; motor eksikleri `raporlar/T1.11-motor-eksikleri.md`) |
| T1.12 | Bağımsız inceleme: Rust ile spesifikasyon arasında sapma, kapsanmayan fonksiyon | Rapor `gelistirme/raporlar/T1.12-motor-inceleme.md`; bulgular kapandı | T1.1–T1.10, T1.14 | Claude | Fable | done (2026-11-12; 5 rapor) |
| T1.17 | T1.11 motor eksikleri ve açık T1.12 bulguları: 03 tipli grist ve çok asitli pH; 04 maya alan adı; 07 C10 "bitti" durumu, Tilt düzeltmesi, 11 §1.3 elle gün çözünürlüklü tetik; 08 §4.3 süzme verimi ve lab oranı; 09 LPA ve uygulanan oran çıktısı, spunding sabiti (`raporlar/T1.11-motor-eksikleri.md`) | Spesifikasyon → referans → vektör → Rust → servis; parite ve uç testleri geçiyor | T1.11 | Claude | Sonnet (yazım), Opus (inceleme) | done (2026-11-13) |
| T1.13 | Aşama 1 kapı kontrolü | Kapı maddeleri sağlandı; sapmalar ADR'de | T1.11, T1.12, T1.17 | Claude | Opus | done (2026-11-13; ADR-0006) |

## Stage 2: Core API and data · CLOSED (2026-11-27, ADR-0014)

Gate:
- The end-to-end tests are green in compose and in kind.
- The critical and important findings of the Fable reviews for the data model (F2.1) and security (F2.2) are closed.
- The merge gate is green: fmt, clippy, test, sqlx check, contract difference, `araclar/tutarlilik.py`.

Scope: the data model also contains the Business tables (plan › Veri modeli). The endpoints cover only the needs of the Home MVP screens. Business endpoints come in stage 6.

Order: decisions (T2.1–T2.3) → data (T2.4–T2.9) → API core (T2.10–T2.18) → domain endpoints (T2.19–T2.28) → support (T2.29–T2.31) → cluster and end-to-end (T2.32–T2.33) → security review and gate. Tasks with the same dependency run in parallel. Each domain task brings its own migration, endpoint tests and OpenAPI difference.

| Task | Description | Done criterion | Depends on | Owner | Model | Status |
|---|---|---|---|---|---|---|
| T2.0 | Aşama 2 iş listesi | Bu tabloda T2.x satırları; DURUM.md güncel | T1.13 | Claude | Opus | done (2026-11-13) |
| T2.1 | Veri modeli (ADR-0007): plan › Veri modeli'ndeki her varlığın tablo tasarımı (Ev MVP ve İşletme birlikte); ortak sütunlar (UUIDv7, oluşturma, güncelleme, silinme zamanı, sürüm numarası); iç birim SI ve sütun adında birim; sabit sütun ile JSONB sınırı; değişmez tarif sürümü ve IngredientUse anlık kopyası; çalışma alanı ayrımı (RLS, `app.workspace_id`); ortak ve çalışma alanına özel kütüphane; Measurement bölümlemesi; outbox ve idempotency tabloları; AuditLog özet zinciri; silme, saklama ve anonimleştirme; veri yeri (bölge) | ADR-0007 ve ER diyagramı (mermaid); her plan varlığının tablo karşılığı ya da gerekçeli dışarıda bırakılışı | T2.0 | Claude | Opus | done (2026-11-13; ADR-0007) |
| T2.2 | API kuralları (ADR-0008): kaynak ve yol adları; hata zarfı ve kod listesi; cursor sayfalama; idempotency anahtarı; ETag ve If-Match ile iyimser kilit, çakışma gövdesi (fark); alan tanımlarının (min, max, birim, zorunlu) veri olarak sunulması; API'de SI birim; yetkiye göre alan gizleme (maliyet); SSE olay biçimi; `/v1` ve `/internal` ayrımı; en düşük uygulama sürümü; OpenAPI geriye uyumsuzluk denetimi | ADR-0008 | T2.0 | Claude | Opus | done (2026-11-13) |
| T2.3 | Kimlik, oturum ve yetki (ADR-0009): Apple ve Google kimlik belirteci doğrulaması (JWKS, iss, aud, nonce); e-posta bağlantısı; opak oturum belirteci (özeti PostgreSQL'de, Redis önbelleği, süreler); web çerez ve CSRF, mobilde Bearer; passkey (WebAuthn) ve yönetici ikinci adımı; hesap bağlama; davet; ortak cihaz PIN'i; izin listesi, rol ve kapsam modeli; Ev modunda paylaşım; servisler arası mTLS yöntemi; sırlar (yerel ve küme) | ADR-0009; kısa tehdit listesi (STRIDE) | T2.1 | Claude | Opus | done (2026-11-13) |
| T2.4 | `db` crate ve göç altyapısı: sqlx (PostgreSQL, çevrimdışı `.sqlx` sorgu önbelleği), `sunucu/migrations/`, `make migrate`, `make sqlx-prepare`, test başına geçici veritabanı, compose'da göç adımı, Kubernetes'te göç Job'ı, CI'da sqlx denetimi | Boş göçle `make up` ve `cargo test -p db` geçiyor; CI'da `cargo sqlx prepare --check` | T2.1 | Claude | Sonnet | done (2026-11-13) |
| T2.5 | Çekirdek göçler: Workspace, User, UserIdentity, passkey, oturum, Role, Permission, RoleAssignment, Invitation, DeviceSession, AuditLog, outbox, idempotency; RLS politikaları; tablo sahibi olmayan, RLS'yi aşamayan uygulama rolü | Göçler temiz; RLS testi: A çalışma alanının bağlamında B'nin satırı okunmuyor ve yazılmıyor; bağlam yoksa satır görünmüyor | T2.3, T2.4 | Claude | Sonnet | done (2026-11-13) |
| T2.6 | Ev MVP alan göçleri: Ingredient, StyleGuide, Style, StyleMapping, Equipment, Profile, Recipe, RecipeVersion, IngredientUse, EditSession, Batch, BatchStep, BatchIngredientActual, Deviation, Measurement (bölümlü), StageEvent, StockLot, StockMovement, PackageType, Notification, NotificationEvent, Reminder, Device, AlertRule, BrewDayPackage, CalcResult, Attachment, ScannedDocument, ImportSource, Subscription, Entitlement, SupportRequest, SupportMessage, SupportGrant, SupportAction | Her tabloda ortak sütunlar ve RLS; RLS testi bütün tabloları tarar, politikası olmayan tablo testi düşürür | T2.5 | Claude | Sonnet | done (2026-11-13) |
| T2.7 | İşletme göçleri: Vessel, VesselOccupancy, Transfer, PackagingRun, QualitySpec, LabResult, Release, CleaningRecord, YeastGeneration; vergi tabloları (ülke, yürürlük tarihi, bant, kademe, indirim) | Aynı RLS taraması geçiyor; uç yok (Aşama 6) | T2.6 | Claude | Sonnet | done (2026-11-13) |
| T2.8 | 09 vergi yöntem zinciri (bant + kademe + indirim) ve yıl başından hacim (`ytd_hl`) taşınması; tablo verisi motora girdi olarak (ADR-0006 açığı) | Spesifikasyon → referans → vektör → Rust → servis; parite ve uç testleri geçiyor | T2.7 | Claude | Sonnet (yazım), Opus (inceleme) | done (2026-11-14) |
| F2.1 | Bağımsız veri modeli incelemesi: ADR-0007 ve göçler; plan › Veri modeli, Eşzamanlı düzenleme, Güvenlik ile karşılaştırma; kısıt, indeks, RLS boşluğu, silme ve anonimleştirme | Rapor `gelistirme/raporlar/F2.1-veri-modeli.md` | T2.7 | Claude | Fable | done (2026-11-14) |
| T2.9 | Fixes for the F2.1 findings | All critical and important findings are closed or are in the decision list; the tests pass | F2.1 | Claude | Sonnet (fix), Opus (review) | done (2026-11-14) |
| T2.10 | API çekirdeği: uygulama durumu, PostgreSQL havuzu, Redis istemcisi; istek bağlamı (kullanıcı, çalışma alanı, izinler) ve işlem başına `SET LOCAL`; hata zarfı; idempotency ara katmanı; If-Match ve ETag; cursor sayfalama; hız sınırı (Redis); istek kimliği ve OpenTelemetry izi; OpenAPI üretimi | ADR-0008'in her kuralına test; Redis kapatılınca API çalışmaya devam ediyor (test) | T2.2, T2.5 | Claude | Sonnet | done (2026-11-13) |
| T2.11 | Giriş: Apple ve Google kimlik belirteci doğrulaması (JWKS önbelleği); e-posta bağlantısı (mailpit); ilk girişte kullanıcı ve çalışma alanı (mod, bölge); oturum, çıkış, her yerden çıkış, oturum listesi; hesap bağlama ve "aynı e-posta başka hesapta"; compose'a sahte OIDC sağlayıcısı | Uçtan uca: sahte Apple ve Google belirteciyle giriş; yanlış aud, iss, süre reddi; e-posta bağlantısı tek kullanımlık ve süreli | T2.3, T2.10 | Claude | Sonnet | done (2026-11-14) |
| T2.12 | Passkey and shared device: WebAuthn registration and verification; mandatory second factor for administrator operations; shared device mode (an administrator opens it, a personal PIN, a lock after a time with no activity), DeviceSession | Endpoint tests; a limit on PIN attempts; an administrator operation without a second factor is rejected | T2.11 | Claude | Sonnet | done (2026-11-14) |
| T2.13 | Authorization: permission list, seed data for the standard roles (plan › Yetki matrisi), edit and copy of roles, RoleAssignment and scope, protection of the last administrator, invitation (single use, 7 days), sharing of recipes and batches in Home mode (editor, viewer), removal of cost fields from responses | One test for each cell of the matrix (role × permission); an unauthorized request gives 403; a record of a different workspace gives 404 | T2.11 | Claude | Sonnet | done (2026-11-14) |
| T2.14 | Denetim kaydı: yalnız eklenebilir tablo (uygulama rolünde UPDATE ve DELETE yok), her satırda öncekinin özeti, zinciri doğrulayan komut; rol, yetki ve kapanmış parti düzeltmeleri yazılır | Satır silinince veya değişince doğrulama hata veriyor (test) | T2.10 | Claude | Sonnet | done (2026-11-13) |
| T2.15 | Olaylar: outbox → NATS JetStream aktarıcısı (worker); tüketici çerçevesi (aynı olay iki kez işlenmez, yeniden deneme, ölü mektup); SSE ucu (çalışma alanı ve kayıt kanalları, podlar arası) | Uçtan uca: kayıt yazılınca SSE istemcisi olayı alıyor; NATS yeniden başlayınca olay kaybolmuyor | T2.10 | Claude | Sonnet | done (2026-11-14) |
| T2.16 | Motor bağlantısı: API'den engine-service gRPC istemcisi; motor sürüm damgası; girdi özetiyle Redis önbelleği; CalcResult kaydı; motor hata kodlarının API hata zarfına eşlenmesi; "Nasıl hesaplandı" ara değerleri | Aynı girdi ikinci kez motora gitmiyor (test); motor kapalıyken anlamlı hata | T2.10 | Claude | Sonnet | done (2026-11-13) |
| T2.17 | Alan tanımları, birim ve metin kataloğu: alan sınırlarının tek kaynağı ve API'den sunulması; birim dönüşüm katsayıları ve TR/EN metin kataloğu tek veri dosyasında, TypeScript ve Swift'e üretim (`make gen`) | Üretilen dosyalar fark denetiminde; sunucu doğrulaması aynı tanımı kullanıyor | T2.2 | Claude | Sonnet | done (2026-11-13) |
| T2.18 | Nesne deposu: tek arayüz (S3, GCS, Azure Blob), compose'a S3 uyumlu yerel servis, kısa süreli imzalı yükleme ve indirme bağlantısı, boyut ve tür sınırı; Attachment ve ScannedDocument uçları | Uçtan uca yükleme ve indirme; süresi geçmiş bağlantı reddi | T2.10 | Claude | Sonnet | done (2026-11-14) |
| T2.19 | Ingredient library and style guides: shared and workspace-specific ingredients, user-added ingredients, ingredient detail; endpoints for style guides, styles and mappings between guides; sample seed data with a separate command (marked "sample"; the source of real library data is a separate decision) | The endpoint tests pass | T2.9, T2.13 | Claude | Sonnet | done (2026-11-20) |
| T2.20 | Equipment, profiles and settings: Equipment (equipment wizard); mash, boil, fermentation, carbonation and water profiles; water analysis report; vessels; workspace and user settings (units, calculation options); last writer wins for each field, with a change history; calibration proposal (engine) | Endpoint tests; a test that restores a value from the history | T2.9, T2.13, T2.16 | Claude | Sonnet | done (2026-11-20) |
| T2.21 | Recipe: Recipe, draft and published immutable version; approval status field for Business mode (the workflow endpoints come in stage 6); IngredientUse snapshot; calculation (T2.16); scaling; comparison of recipes and versions; EditSession soft lock (SSE); folders, tags, favorites; mead, cider, seltzer; mead honey percentage and `est_cells_ml` (ADR-0006 gaps) | End-to-end: recipe → calculation → publish → the old version does not change; a save with an old version gives 412 and the difference | T2.19, T2.20 | Claude | Sonnet | done (2026-11-20) |
| T2.22 | Batch and brew day package: start a batch from a recipe; BatchStep; actual quantity and deviation; measurements (safe to repeat with the client UUID); stage transitions; batch summary (plan against actual); calendar and backward planning; brew day package (steps, timers and targets that the server calculates; signed and with an expiry time); acceptance of measurements with the package ID | End-to-end: the same measurement sent two times gives one record; a measurement with the package of a different batch is rejected; an expired package is rejected | T2.21 | Claude | Sonnet | done (2026-11-20) |
| T2.23 | Stock and shopping list: StockLot (one lot in Home mode), movement ledger, totals calculated from movements, warning for negative stock, goods receipt and stock count, deduction for a batch, shopping list from a recipe | Endpoint tests; two movements at the same time give the correct total | T2.22 | Claude | Sonnet | done (2026-11-20) |
| T2.24 | Notifications: Notification, NotificationEvent, Reminder; three priorities, quiet hours, daily summary; push (fake APNs) and email (mailpit) in the worker; notification inbox and actions (approve, postpone, ignore); the worker evaluates alert rules (AlertRule) on measurement events | End-to-end: measurement → rule → notification → request to the fake APNs | T2.15, T2.22 | Claude | Sonnet | done (2026-11-20) |
| T2.25 | Device data intake (ingest): device registration and pairing; one address for each device, with a key that you can rotate; iSpindel, RAPT and Brewfather "Custom Stream" formats (the format is to be verified); rejection of values out of range; write to NATS, the worker writes to Measurement | End-to-end: iSpindel request → measurement → SSE; after a key rotation, the old key is rejected | T2.15, T2.22 | Claude | Sonnet | done (2026-11-21) |
| T2.26 | Import and export: BeerXML and BeerJSON import (safe XML parser, size limit); import of recipes and equipment from a Brewfather JSON backup and from `.bsmx`, with source and calculated OG, IBU and color side by side; mapping decisions (ImportSource); export (BeerJSON, BeerXML, escaped CSV, JSON, ZIP) as a worker job; the full export puts measurements in one CSV file for each batch and year, and gives a download link with an expiry time; a user can start one full export per 24 hours; a batch export (one batch or a date range of measurements) has no such limit; `.bsmx` if a sample file is available | Round-trip test with sample files; an XML bomb and external entities are rejected; a second full export in 24 hours is rejected; a batch export works after a full export | T2.18, T2.21 | Claude | Sonnet | done (2026-11-21) |
| T2.27 | Global search: PostgreSQL full-text search (recipes, batches, ingredients, stock), limited to the workspace | Endpoint test; no results from a different workspace | T2.23 | Claude | Sonnet | done (2026-11-20) |
| T2.28 | Subscription: Subscription, Entitlement; a full trial for 1 month, after that the paid features are read-only, export is always available; App Store Server Notifications v2 (JWS verification) in ingest; interface for a web payment provider (decision O6 selects the provider) | End-to-end: renewal, refund, trial end; the same notification is not processed two times | T2.13, T2.15 | Claude | Sonnet | done (2026-11-20). Crate `billing`, migration 0130, ingest JWS verification, worker consumer, `GET /v1/workspaces/{ws}/subscription`, guard `require_paid_write`. Web provider: Paddle adapter not written yet (trait and fake only). |
| T2.29 | Support requests: SupportRequest, SupportMessage, request statuses, alias identity, attachments in the object store; sync with a GitHub App (outbox → GitHub API, retry at the rate limit); GitHub webhook in ingest (signature, repeated delivery); the `/yanit` rule and the list of the support team; ignore own comments; edits and deletes go to the other side; fake GitHub (wiremock) | End-to-end: request → Issue; `/yanit` → notification; an internal note does not go to the customer; when GitHub is not available, the request opens and syncs later | T2.15, T2.18, T2.24 | Claude | Sonnet | done (2026-11-20) |
| T2.30 | Support panel endpoints (`/internal`): account search and summary, subscription re-verification and correction with a reason, session close, device key rotation, deletion requests; SupportGrant (72 hours, reason, request number), emergency access with two approvals; SupportAction audit log and "Destek erişim geçmişi" for the customer | A request for content without a grant gives 403; access stops when the grant expires; each operation is in the audit log | T2.13, T2.14, T2.29 | Claude | Sonnet | done (2026-11-20) |
| T2.41 | Paddle adapter for `WebPaymentProvider` (signature, price map, status mapping) and the connection in `ingest::state_from_env`; refunds come later (Issue #266) | Unit tests and the `ingest` test `tests/paddle.rs` pass in CI; Claude reviews and merges | T2.28 | ChatGPT | – | done (2026-11-25, #266) |
| T2.42 | Add a cursor to the trash list (`GET .../trash`): same cursor format as the other list endpoints (ADR-0008) (Issue #282) | Endpoint tests for cursor pages, filter and bad cursor pass in CI; Claude reviews and merges | T2.37 | ChatGPT | – | done (2026-11-25, #282) |
| T2.43 | Worker job: set a `resolved` support request to `closed` after 14 days; migration 0240 (Issue #283) | Worker tests pass in CI: 15 days closes, 13 days stays, a rerun changes nothing; Claude reviews and merges | T2.29 | ChatGPT | – | done (2026-11-25, #283) |
| T2.44 | Worker job: delete the object bytes of deleted uploads and of incomplete uploads older than 24 hours; migration 0241 (Issue #286; do after #283) | Worker tests pass in CI: deleted and incomplete uploads lose their bytes, a legal hold stays, a failed delete is tried again; Claude reviews and merges | T2.18, T2.43 | ChatGPT | – | done (2026-11-25, #341) |
| T2.45 | Brew day package key rotation: verify with the current key and with previous keys; setting `BREW_PACKAGE_PREVIOUS_KEY_IDS` (Issue #284) | Tests pass in CI: a package of the previous key verifies, an unknown key id is rejected; Claude reviews and merges | T2.22 | ChatGPT | – | done (2026-11-25, #284) |
| T2.46 | A finding of the weekly audit digest check writes the outbox event `audit.digest_check_failed` (Issue #287; do after #286) | Worker tests pass in CI: a finding writes one event with counters only, a clean run writes none; Claude reviews and merges | T2.39, T2.44 | ChatGPT | – | done (2026-11-26) |
| T2.47 | Sentry for the server in `common` behind a feature: off without `SENTRY_DSN`; `before_send` filter and its test (ADR-0010; Issue #285) | Filter tests pass in CI; the build without the feature has no `sentry` dependency; Claude reviews the new dependencies and merges | T2.10 | ChatGPT | – | done (2026-11-26) |
| T2.48 | E2E test for the SSE events of a recipe edit session (Issue #366) | The new e2e scenario passes in CI; class A | T2.22 | ChatGPT | – | done (2026-11-26, #366) |
| T2.49 | Trash restore writes the audit row `trash.restore` and the outbox event `<kind>.restored` (Issue #367) | api and db tests pass in CI; Claude reviews and merges | T2.37 | ChatGPT | – | done (2026-11-26, #367) |
| T2.50 | Compose service `seed` (profile `seed`) and `make seed` for the sample library (Issue #368) | `make seed` runs two times with exit 0; Claude reviews and merges | T2.19 | ChatGPT | – | done (2026-11-26, #368, PR #380) |
| T2.51 | Allow-list of App Store product IDs (`APPLE_PRODUCT_IDS`) in the ingest Apple webhook; G1 follow-up (Issue #381) | `ingest` tests pass in CI: unknown product ID changes no row; Claude reviews (Opus) and merges | T2.34 | ChatGPT | – | done (2026-11-26, #381) |
| T2.52 | E2E scenarios for idempotency replay and ETag conflict (Issue #382) | New e2e tests pass in CI; class A | T2.33 | ChatGPT | – | done (2026-11-26, #382) |
| T2.53 | Step-up for `roles_create`, `roles_copy`, `roles_delete`, `members_remove`, `assignments_delete`, `invitations_revoke`; G2 follow-up (Issue #383) | api and e2e tests pass in CI; Claude reviews (Opus) and merges | T2.34, PR #380 | ChatGPT | – | done (2026-11-26, #383) |
| T2.54 | Rate limit for the device ingest endpoint (F2.2 G5; Issue #395) | `ingest` tests pass in CI: over the limit gives 429; keys not in logs; Claude reviews and merges | T2.34 | Claude | – | done (2026-11-27, #395, PR #420) |
| T2.55 | Staff check inside `support_find_workspace` (F2.2 G7; migration; Issue #396) | api and db tests pass in CI; Claude reviews (Opus) and merges | T2.34 | Claude | – | done (2026-11-27, #396, PR #421, migration 0253) |
| T2.56 | E2E scenarios for kegs and containers (K2 flows; Issue #397) | New e2e tests pass in CI; class A | K2 | ChatGPT | – | done (2026-11-27, #397, PR #408) |
| T2.57 | E2E scenarios for modules, trial and read-only state (P1 flows; Issue #398) | New e2e tests pass in CI; class A | P1 | ChatGPT | – | done (2026-11-26, #398, PR #400) |
| T2.58 | E2E for trial end and read-only module state (follow-up of T2.57) | `make e2e` is green in CI; the mechanism that ends a trial cannot run in production | T2.57, #390 B2 | ChatGPT | Sonnet | todo (#403, ready) |
| T2.59 | Package types API: `GET` and `POST /v1/workspaces/{id}/package-types` (K2 gap) | Endpoint tests pass; the OpenAPI difference has only the two operations | K2 | Claude | Sonnet | done (2026-11-27, #409, PR #412) |
| T2.60 | Check the intermediate certificate marker in a two-certificate App Store chain (F2.2 G4) | `cargo test -p ingest`: a two-certificate chain without the marker fails | T2.34 | Claude | Sonnet | done (2026-11-27, #413, PR #417) |
| T2.32g | Istio AuthorizationPolicy for PostgreSQL, Redis and NATS (Issue #369, part of #197) | The kind smoke test passes with the new negative checks; Claude reviews and merges | T2.32e | ChatGPT | – | done (2026-11-27) |
| T4.a | iOS API client from the OpenAPI contract (swift-openapi-generator; three Apple packages approved) (Issue #370) | Unit tests and macOS CI are green; Claude reviews and merges | – | Claude | – | done (2026-11-27, #370, PR #424) |
| T2.31 | Account deletion and data export request: deletion from the app, revocation of the Apple token, retention period and restore, anonymization, deletion or anonymization of GitHub Issues, full export (ZIP) | End-to-end: no data of the deleted account remains; the audit log is anonymous; the chain verification passes | T2.26, T2.30 | ChatGPT (#198: parts #212–#217); Claude (review, test, merge) | Sonnet | done (2026-11-26) |
| T2.32 | Cluster: PostgreSQL, Redis, object store and fake services in kind (local overlay); migration Job; mTLS between services (ADR-0009); network policies (only the API connects to the engine; `/internal` only from the OIDC reverse proxy); oauth2-proxy; secrets; Terraform modules for Cloud SQL, Memorystore, Cloud Storage | `make kind-test` passes with the new checks (`/internal` from the public ingress gives 404); `terraform validate` is clean | T2.11, T2.15, T2.30 | ChatGPT (#197: parts #203–#208); Claude (review, test, merge) | Sonnet | done (2026-11-27) |
| T2.37 | Recently deleted: the user lists soft-deleted recipes, batches, ingredients and equipment of the last 30 days and restores them; after 30 days the worker deletes them permanently (F2.1 decision 9 applies) | Endpoint tests: restore within 30 days; no restore after 30 days; a record of a different workspace gives 404 | T2.22 | Claude | Sonnet | done (2026-11-20) |
| T2.38 | Engine unit constants: change the engine to the exact unit definitions of the catalog (`sozlesme/katalog`); keep the tolerance of the BeerSmith parity tests; also allow the split of conversion and lauter efficiency in Home mode (equipment field, advanced setting) | All parity tests pass; a test shows that the engine and `common::catalog` give the same conversion factors | T2.17 | Claude | Sonnet | done (2026-11-20) |
| T2.39 | Worker jobs for the F2.1 decisions: daily digest of the audit chain heads to a retention-locked area of the object store, weekly verification against these digests (decision 12); clear the payload of unprocessed external events after 30 days and delete event IDs after 13 months (decision 11) | Tests: the daily job writes a digest; a changed chain row makes the weekly check fail; the retention job clears and deletes the correct rows | T2.14, T2.15, T2.18 | Claude | Sonnet | done (2026-11-20) |
| T2.40 | Equipment efficiency split in the API: two nullable equipment columns (`conversion_efficiency_pct`, `lauter_efficiency_pct`), the settings endpoints for them (T2.20), and the recipe calculation request (T2.21, `build_request` TODO) | Endpoint test: equipment with both values gives the product efficiency in the calculation; one value alone is rejected | T2.21, T2.38 | Claude | Sonnet | done (2026-11-20) |
| T2.36 | Visual architecture flow diagram (high level): clients (iPhone/iPad, web), edge, application services (api, worker, ingest), calculation engine, data (PostgreSQL, Redis, NATS, object store) and all external connections, each as a box with one arrow (Apple sign-in, Google sign-in, App Store, APNs, email, GitHub, measurement devices iSpindel/RAPT, web payment provider, observability/OTel, cloud); for each connection only the name and the direction, no details | A mermaid diagram in `belgeler/mimari-akis.md` and a published visual page (Artifact); agrees with plan › Platformlar ve mimari; each external connection shows one time | T2.32 | ChatGPT (#194, draft); Claude (last check after T2.32, Artifact) | Sonnet | done (2026-11-27) |
| T2.33 | End-to-end suite: scenarios on compose (full flow of a Home user: login → setup → recipe → batch → package → measurement → notification → support request; isolation of two workspaces); `make e2e` and a CI workflow; a short suite in kind | `make e2e` is green locally and in CI | T2.21–T2.32 | ChatGPT (#195, scenarios done: `gelistirme/raporlar/T2.33-senaryolar.md`; #229, test code); Claude (review, test, merge) | Sonnet | done (2026-11-24, PR #230) |
| F2.2 | Independent security review: identity, session, authorization, workspace isolation, signature of the brew day package, webhook signatures, support panel access, file upload, import | Report `gelistirme/raporlar/F2.2-guvenlik.md` | T2.33 | Claude (#364) | Fable | done (2026-11-26) |
| T2.34 | Fixes for the F2.2 findings | All critical and important findings are closed or are in the decision list; the end-to-end suite is green | F2.2 | Claude | Sonnet (fix), Opus (review) | done (2026-11-26) |
| T2.35 | Stage 2 gate check | The gate items are satisfied; deviations are in an ADR | T2.9, T2.34, T2.36 | Claude | Opus | done (2026-11-27, ADR-0014) |

## Stages 3–6

The stages are in plan › Geliştirme düzeni and plan › Yol haritası. Opus writes the task list of a stage when the previous stage closes.

| Stage | Content | Gate |
|---|---|---|
| 3 Web | Home MVP screens | Playwright, accessibility |
| 4 iOS | SwiftUI, brew day package, Live Activity; the iPhone shows only the screens in plan › iPhone kapsamı | macOS CI is green; device test (Oğuz) |
| 5 Home MVP beta | Cloud setup, beta | Interviews and golden test are complete; independent security review |
| 6 Business | Business screens | Step by step |

## UI quality program: UIQ (stages 3–4)

Decision (Oğuz, 2026-11-27; ADR-0016): the GRISTORA React UI Kit, the GRISTORA SwiftUI Kit, adaptive layout for web and iPad, and the Visual Quality Gate in CI. The full task table, the standards and the done criteria are in `gelistirme/uiq/README.md` › 9. This table gives only the status and the issue.

Rules:
- New screens of stage 3 use Kit parts when the part exists. Do not add a screen-specific copy of a Kit part.
- Do not add a dependency other than `react-aria-components@1.22.1`.
- Visual baselines are class C. Only Claude commits them.

| Task | Owner | Model | Issue | Status |
|---|---|---|---|---|
| UIQ-01 audit | Claude | Sonnet, Opus | – | done (2026-11-27) |
| UIQ-02 Apple patterns | Claude | Opus | – | done (2026-11-27) |
| UIQ-03a structural tokens | Claude | Sonnet | – | done (2026-11-27) |
| UIQ-03b chart palette, design system README and Artifact | Claude | Opus | – | todo |
| UIQ-04 React Kit skeleton and gallery | ChatGPT | – | ISSUE04 | todo |
| UIQ-05 SwiftUI Kit skeleton | Claude | Sonnet | – | active |
| UIQ-06 web adaptive AppShell, `check-css.mjs` | ChatGPT | – | ISSUE06 | blocked (UIQ-04, UIQ-08) |
| UIQ-07 iPad adaptive root | Claude | Sonnet | – | todo |
| UIQ-08 React Aria pilot | ChatGPT | – | ISSUE08 | blocked (UIQ-04) |
| UIQ-09 React Aria rollout | ChatGPT | – | – | todo |
| UIQ-10 web geometry and screenshot tests | ChatGPT | – | ISSUE10 | blocked (UIQ-04) |
| UIQ-11 iPad screenshot and audit tests | Claude | Sonnet | – | todo |
| UIQ-12 CI Visual Quality Gate | Claude | Sonnet | – | todo |
| UIQ-13 move screens to the Kit | ChatGPT (web), Claude (iOS) | Sonnet | – | todo |
| UIQ-14 theme, language, accessibility audit | ChatGPT | – | – | todo |
| UIQ-15 fix found defects | ChatGPT, Claude | Sonnet | – | todo |
| UIQ-16 final quality and performance review | Claude | Opus | – | todo |

## Product modules (stage 5)

Decision (Oğuz, 2026-11-20): the product has modules (plan › Modüller): Core, Devices, Stock, Production, Quality, Cost and reports, Tax and compliance, Team, Connections. Each module can be on or off for each workspace. The tier matrix (O8) uses the modules. The module borders keep the product out of ERP.

| Task | Description | Done criterion | Depends on | Stage | Owner | Model | Status |
|---|---|---|---|---|---|---|---|
| P1 | Module switches: a module list in code (one source), the module of each endpoint module in `api/src/modules.rs`, a check that gives 403 `module_disabled` when the module is off, module state from `entitlements`, the module list in `/v1/meta` | Endpoint tests: an endpoint of a module that is off gives 403; export is always available; the OpenAPI document shows the module of each operation | T2.28, T2.35 | 5 | ChatGPT | – | done (2026-11-26, #356) |
| P2 | Tier matrix draft with the modules (Home and Business tiers, limits for devices, users and storage; no limit on export); agree with the pricing of the marketing site (`web-stratejisi/45-*`) | Draft in plan › Deneme süresi; O8 can decide | P1 | 5 | ChatGPT (#196, draft done: `gelistirme/raporlar/P2-kademe-matrisi.md`); Claude (review, copy into the plan) | Opus | todo |
| P3 | Module state in the clients: settings screen (modules of the workspace), hidden navigation for modules that are off, an upgrade message; update the concept, the screen list and the design system | Playwright and iOS UI tests; `araclar/tutarlilik.py` is clean | P1 | 5 | ChatGPT (#326: web step 1); Claude (step 2, iOS, concept) | Sonnet | active (web done: #326, #392; iOS step active) |

## Keg tracking and the field view (stage 6, Business version 2)

Decision (Oğuz, 2026-11-20): Business mode gets a light keg and cask tracking feature in the Production module (plan › Ambalaj ve kap türleri). Each returnable container has a serial number and a QR label. A scan changes the status. A fill links the container to the packaging run and the lot.

Rules:
- Do not add customers, deposits or overdue alerts. These are ERP functions and are out of scope.
- The status "out" has a free text note only. Do not link it to a customer record.
- Do not put the workspace ID or other internal IDs in the QR code. Use an opaque code.
- The QR code of a tank and a lot uses the same opaque code (`ScanCode`).
- The field view is on the iPhone only. The device split is a user interface rule. The API does not check the device. The server checks the permission.
- Do not start these tasks before stage 6.

| Task | Description | Done criterion | Depends on | Stage | Owner | Model | Status |
|---|---|---|---|---|---|---|---|
| K1 | Data model for `Container`, `ContainerEvent` and `ScanCode` (plan › Veri modeli): serial number, opaque QR code for containers, tanks and lots, `PackageType`, status (clean, filled, out, dirty, maintenance, retired), last fill (packaging run row and lot), event log (scan, user, time, old and new status, note); migration, row level security, and the permissions "field view", "change container status" and "manage containers and labels" with the role "Saha" (plan › Yetki matrisi) | Migration tests; a container of a different workspace is not visible; a dirty container cannot be filled | T2.35 | 6 | ChatGPT | – | done (2026-11-25, #337) |
| K2 | API endpoints: resolve a scanned code to its target (container, tank or lot) with the permitted actions; list with status counts, detail, bulk create with serial numbers, status change from a scan and from a batch scan, retire; a fill from a packaging run row; recall query from a lot to the container list; label PDF for containers, tanks and lots (QR code, serial number, size) | Endpoint and contract tests; an unknown or revoked code gives 404; a user without the permission does not get the action; the recall query gives each filled container of the lot; the OpenAPI document shows the Production module | K1 | 6 | Claude (#363) | Sonnet | done (2026-11-26) |
| K3 | Web screen "Fıçılar" (concept pane `kg`): list, status counts, detail with the event log, label print | Playwright and accessibility tests | K2 | 6 | Claude | Sonnet | done (2026-11-27, #365, PR #425) |
| K4 | iOS field view on the iPhone (concept pane `sh`): one scan screen for tank, lot and container codes, a card with the permitted actions, batch scan, the tabs for the role "Saha", Control Center and Siri shortcuts (App Intents); the iPad shows the list and the detail only | iOS UI tests on macOS CI; device test (Oğuz) | K2 | 6 | Claude | – | active (#391: steps 1–2 done, PR #440, #443; steps 3–4 open) |

## Bring your own agent: MCP (stage 5 and Home version 2)

Decision (Oğuz, 2026-11-20): the user can connect an own AI agent (Claude, ChatGPT or another MCP client) through a remote MCP server. We do not run a language model. The design is in plan › Platformlar ve mimari › API ve istemci iletişimi.

Rules:
- The MCP server is a thin layer on the `/v1` API. Do not write business rules again in it.
- The agent does not write calculated numbers. The engine calculates them.
- The agent cannot publish, approve or delete.
- Do not start these tasks before the given stage.

| Task | Description | Done criterion | Depends on | Stage | Owner | Model | Status |
|---|---|---|---|---|---|---|---|
| M1 | ADR for OAuth 2.1 and MCP: authorization server (PKCE, dynamic client registration as the MCP specification requires), scopes, consent screen, token lifetime and revocation; the list of MCP tools and their scopes; audit entries "via MCP"; rate limits; the risk of prompt injection through user text in tool results; the same OAuth base for the Business open API | `adr/00xx-mcp-ve-oauth.md`; plan and ADR agree | T2.35 | 5 | Claude | – | done (2026-11-27, #389, PR #423; ADR-0015 accepted, PR #432) |
| M2 | OAuth 2.1 authorization server in the API: authorization code with PKCE, client registration, scopes, consent, refresh and revocation; "Connected apps" endpoints | Endpoint tests: a token without a scope gives 403; a revoked token gives 401; a token of a different workspace gives 404 | M1 | Home v2 | Claude | Sonnet | done (2026-11-27, PR #441, migrations 0257–0259) |
| M3 | Remote MCP server (Rust, Streamable HTTP): tools for search and read (recipes, batches, fermentation data, stock), calculation through the engine, draft recipe, add measurement, shopping list; each tool calls the `/v1` services | Contract tests for each tool; a publish, approve or delete request is not possible; each write call writes an audit row (ADR-0015) | M2 | Home v2 | Claude | Sonnet | todo |
| M4 | "Connected apps" screen on web and iOS (Settings): list, scopes, last use, remove; update the concept, the screen list and the design system | Playwright and iOS UI test; `araclar/tutarlilik.py` is clean | M2 | Home v2 | Claude | Sonnet | todo |
| M5 | Independent security review of OAuth and MCP (token handling, scope checks, prompt injection, data leaks between workspaces) | Report `gelistirme/raporlar/M5-mcp-guvenlik.md`; critical and important findings are closed | M3, M4 | Home v2 | Claude | Fable | todo |

## Brand: GRISTORA in the product (stages 3–5)

Decision (Oğuz, 2026-11-20): the product name is GRISTORA. The domain is gristora.com. The app (iOS and web app) has its own design system. The marketing site (`marketing-site/`) has a different design. Only the name, the logo, the app icon and the brand accent color are shared. The approved logo files are in `marketing-site/public/brand/` (`marketing-site/BRAND-ASSETS.md`).

Rules:
- Do not change `marketing-site/` from these tasks. It is a separate work stream (`OTURUM.md`).
- Do not rename internal identifiers (crates, database names, `demleme` roles, API paths) unless G1 decides it. A rename gives risk and no value to the user.
- Do a task in the stage that is given in the table. Do not start these tasks before that stage.

| Task | Description | Done criterion | Depends on | Stage | Owner | Model | Status |
|---|---|---|---|---|---|---|---|
| G1 | Brand ADR for the product: the name in each place (app, App Store, email sender, OpenAPI title, web title); the domains (`app.`, `api.`, `auth.`, `mail.` under gristora.com); the shared brand elements (logo, app icon, accent color) and how the app design system uses the accent color (contrast check against the dark high-contrast theme); internal identifiers stay or change | `adr/0012-urun-markasi.md` (0010 is Sentry); the design system and the plan agree | – | 3 (start) | Claude | Opus | done (2026-11-25) |
| G2 | Scan for the old product name: list each user-visible "Demleme" text in `web/`, `ios/`, `sozlesme/`, email templates and the concept | A list in `gelistirme/raporlar/G2-ad-taramasi.md` (file, line, text) | G1 | 3 | ChatGPT (#193) | – | done (2026-11-22) |
| G3 | Product documents: change the product name to GRISTORA in the concept (`arayuz/`), the design system (`tasarim-sistemi/` and its Artifact) and the plan title; add the logo and the accent color tokens to the design system | `araclar/tutarlilik.py` is clean; the concept has no overflow at 1280 px and 400 px | G1, G2 | 3 | ChatGPT (#385) | – | done (2026-11-27) |
| G4 | Server texts: the email templates (sender name GRISTORA, address `no-reply@gristora.com`), the OpenAPI title; run `make gen` | The generated files are in the difference check; the email test passes | G1, G2 | 3 | ChatGPT (#351) | – | done (2026-11-26) |
| G5 | Web app brand: the logo in the app header (from `gristora-symbol.svg` and the wordmark), favicon, page title, web manifest; use the tokens of the app design system, not the styles of the marketing site | Playwright and accessibility checks pass; screenshots at 1280 px and 400 px | G3, G4 | 3 | ChatGPT | – | done (2026-11-27, #352, PR #429) |
| G6 | iOS brand: display name GRISTORA, AppIcon (light, dark, tinted; from the logo symbol, Apple icon rules), launch screen, accent color in the asset catalog | macOS CI is green; device check (Oğuz) | G3, O3 | 4 | Claude | Sonnet | done (2026-11-27, PR #431) |
| G7 | Domains and configuration: DNS for gristora.com in Terraform (Cloud DNS); ingress hosts; redirect URIs for Apple and Google sign-in; cookie domain and CORS; SPF, DKIM, DMARC for email; the universal link file (`apple-app-site-association`) | `terraform validate` is clean; `make kind-test` passes with the new hosts | G1, O4, O7 | 5 | Claude | Sonnet | todo |
| G8 | Brand consistency check: the app (web and iOS screenshots) and the marketing site use the same name, logo and icon; the site does not claim a feature that the app does not have | Report `gelistirme/raporlar/G8-marka-tutarliligi.md` | G5, G6 | 5 | Claude | Sonnet | todo |

## Subscription and payment (stages 3–4)

Decision (Oğuz, 2026-11-25): a Home workspace can subscribe in the App Store (in-app purchase) or on the web (Paddle). A Business workspace subscribes and manages its subscription only on the web (Paddle). The design is in plan › Abonelik ve ödeme.

Rules:
- A workspace has one active payment source at a time. Do not show a purchase in the other channel.
- Do not show a Business price or a Business purchase button in the iOS app.
- Keep the subscription state and the entitlement state on the server only. The client shows them.
- Do not use an App Store introductory offer. The server trial (1 month) is the only trial.

| Task | Description | Done criterion | Depends on | Stage | Owner | Model | Status |
|---|---|---|---|---|---|---|---|
| B1 | ADR for the payment sources: App Store for Home, Paddle for Home and Business, one active source for each workspace, App Store rules 3.1.1 and 3.1.3 (to be verified), plan key and seat count, change from Home to Business | `adr/0013-odeme-kaynaklari.md` | T2.41, P1 | 3 | ChatGPT (#384) | – | todo |
| B2 | Plan key and seats: the entitlement gets a plan (Home, Business and tier) and a seat count instead of one `PAID_FEATURE`; the module set comes from the plan; an audit row for each change; one migration | db and endpoint tests; P1 tests stay green | B1 | 3 | Claude | – | done (2026-11-27, #390, PR #422, migration 0254) |
| B3 | Billing endpoints: `GET /v1/billing`, `POST /v1/billing/checkout` (web only), `POST /v1/billing/portal`, `POST /v1/billing/apple/sync` (StoreKit 2 transaction JWS); 409 when the other source is active | Endpoint tests; the contract check is green | B2 | 3 | Claude | – | done (2026-11-27, #390, PR #422, migration 0255) |
| B4 | Ingest: map the App Store and Paddle events to one subscription row for each workspace; detect a second active source and open a support item | ingest tests for both sources and for a double source | B2 | 3 | Claude | – | done (2026-11-27, #390, PR #422) |
| B5 | Web screen Settings › Subscription: plan and state, Paddle checkout, plan change, Paddle customer portal, cancel; the content security policy allows the Paddle domains | Playwright and accessibility tests with a Paddle sandbox stub | B3, G5 | 3 | ChatGPT | Sonnet | todo (#427) |
| B6 | iOS Settings › Subscription: StoreKit 2 subscription view for Home, restore, manage (Apple sheet); status only for Business; `appAccountToken` is the workspace ID | macOS CI is green; sandbox purchase on a device (Oğuz) | B3, O3 | 4 | Claude | Sonnet | todo |
| B7 | Notifications for billing: the trial ends in 7 days and in 1 day, a payment failed, the subscription ended | Worker tests for each notification | B2 | 3 | Claude | – | done (2026-11-27, #390, PR #422, migration 0256) |

## Tasks for Oğuz (parallel)

| Task | Description | Necessary for | Status |
|---|---|---|---|
| O1 | User interviews (`plan/gorusme.md`) | Stage 5 gate | todo |
| O2 | Golden test numbers from the BeerSmith screen (`formuller/ALTIN-TEST.md`) | Stage 5 gate | todo |
| O3 | Apple Developer account, macOS CI (GitHub Actions macOS or Xcode Cloud) | T0.9, stage 4 | active: macOS CI runs on GitHub macOS machines; the Apple Developer account and the device test are not done |
| O4 | Google Cloud organization and billing | Stage 5 | active (owner, 2026-11-25) |
| O5 | Delete the 7 old branches on GitHub | – | done (2026-11-27, setting on; #255 ChatGPT deletes the old branches) |
| O6 | Select a web payment provider (subscriptions outside the App Store). Decision: Paddle (merchant of record). Paddle collects and pays US sales tax and UK VAT. | T2.28 | done (2026-11-20) |
| O9 | Select a tool for error and crash reports. Decision: Sentry (ADR-0010). Traces, metrics and logs stay in OpenTelemetry. | Stages 3–5 | done (2026-11-24) |
| O8 | Approve the tier matrix and the prices (Claude writes the draft before the beta, task P2) | Stage 5 | todo: trial decided (Home 3 months, Business 1 month, 2026-11-25); prices wait for #358 |
| O7 | Buy the gristora.com domain (after the trademark and domain review, Issue #146) | G7 | done (2026-11-27, gristora.com is bought) |
| O10 | Open the Paddle account and complete the seller verification; give the API key and the webhook secret as cluster secrets (not in the repository) | B3, B5 | active (owner, 2026-11-25) |
| O11 | App Store Connect: subscription group and product IDs for the Home plans | B6 | todo |
