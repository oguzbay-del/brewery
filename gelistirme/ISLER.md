# Task list

This is the only place to track development. When a status changes, update this file and `DURUM.md` in the same commit. Write new and changed rows in ASD-STE100 (root `CLAUDE.md` › Yazım standardı). Do not rewrite the Turkish text of closed stages and done tasks.

**Status:** `todo` · `active` · `done (YYYY-MM-DD)` · `blocked (reason)`. The commit of a task is the commit with a message that starts with the task number (`git log --grep '^T0.2:'`).
**Owner:** `Claude` · `Oğuz` (product owner)
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

## Stage 2: Core API and data · ACTIVE

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
| T2.19 | Ingredient library and style guides: shared and workspace-specific ingredients, user-added ingredients, ingredient detail; endpoints for style guides, styles and mappings between guides; sample seed data with a separate command (marked "sample"; the source of real library data is a separate decision) | The endpoint tests pass | T2.9, T2.13 | Claude | Sonnet | todo |
| T2.20 | Equipment, profiles and settings: Equipment (equipment wizard); mash, boil, fermentation, carbonation and water profiles; water analysis report; vessels; workspace and user settings (units, calculation options); last writer wins for each field, with a change history; calibration proposal (engine) | Endpoint tests; a test that restores a value from the history | T2.9, T2.13, T2.16 | Claude | Sonnet | todo |
| T2.21 | Recipe: Recipe, draft and published immutable version; approval status field for Business mode (the workflow endpoints come in stage 6); IngredientUse snapshot; calculation (T2.16); scaling; comparison of recipes and versions; EditSession soft lock (SSE); folders, tags, favorites; mead, cider, seltzer; mead honey percentage and `est_cells_ml` (ADR-0006 gaps) | End-to-end: recipe → calculation → publish → the old version does not change; a save with an old version gives 412 and the difference | T2.19, T2.20 | Claude | Sonnet | todo |
| T2.22 | Batch and brew day package: start a batch from a recipe; BatchStep; actual quantity and deviation; measurements (safe to repeat with the client UUID); stage transitions; batch summary (plan against actual); calendar and backward planning; brew day package (steps, timers and targets that the server calculates; signed and with an expiry time); acceptance of measurements with the package ID | End-to-end: the same measurement sent two times gives one record; a measurement with the package of a different batch is rejected; an expired package is rejected | T2.21 | Claude | Sonnet | todo |
| T2.23 | Stock and shopping list: StockLot (one lot in Home mode), movement ledger, totals calculated from movements, warning for negative stock, goods receipt and stock count, deduction for a batch, shopping list from a recipe | Endpoint tests; two movements at the same time give the correct total | T2.22 | Claude | Sonnet | todo |
| T2.24 | Notifications: Notification, NotificationEvent, Reminder; three priorities, quiet hours, daily summary; push (fake APNs) and email (mailpit) in the worker; notification inbox and actions (approve, postpone, ignore); the worker evaluates alert rules (AlertRule) on measurement events | End-to-end: measurement → rule → notification → request to the fake APNs | T2.15, T2.22 | Claude | Sonnet | todo |
| T2.25 | Device data intake (ingest): device registration and pairing; one address for each device, with a key that you can rotate; iSpindel and RAPT formats; rejection of values out of range; write to NATS, the worker writes to Measurement | End-to-end: iSpindel request → measurement → SSE; after a key rotation, the old key is rejected | T2.15, T2.22 | Claude | Sonnet | todo |
| T2.26 | Import and export: BeerXML and BeerJSON import (safe XML parser, size limit), mapping decisions (ImportSource); export (BeerJSON, BeerXML, escaped CSV, JSON, ZIP) as a worker job; `.bsmx` if a sample file is available | Round-trip test with sample files; an XML bomb and external entities are rejected | T2.18, T2.21 | Claude | Sonnet | todo |
| T2.27 | Global search: PostgreSQL full-text search (recipes, batches, ingredients, stock), limited to the workspace | Endpoint test; no results from a different workspace | T2.23 | Claude | Sonnet | todo |
| T2.28 | Subscription: Subscription, Entitlement; a full trial for 1 month, after that the paid features are read-only, export is always available; App Store Server Notifications v2 (JWS verification) in ingest; interface for a web payment provider (decision O6 selects the provider) | End-to-end: renewal, refund, trial end; the same notification is not processed two times | T2.13, T2.15 | Claude | Sonnet | todo |
| T2.29 | Support requests: SupportRequest, SupportMessage, request statuses, alias identity, attachments in the object store; sync with a GitHub App (outbox → GitHub API, retry at the rate limit); GitHub webhook in ingest (signature, repeated delivery); the `/yanit` rule and the list of the support team; ignore own comments; edits and deletes go to the other side; fake GitHub (wiremock) | End-to-end: request → Issue; `/yanit` → notification; an internal note does not go to the customer; when GitHub is not available, the request opens and syncs later | T2.15, T2.18, T2.24 | Claude | Sonnet | todo |
| T2.30 | Support panel endpoints (`/internal`): account search and summary, subscription re-verification and correction with a reason, session close, device key rotation, deletion requests; SupportGrant (72 hours, reason, request number), emergency access with two approvals; SupportAction audit log and "Destek erişim geçmişi" for the customer | A request for content without a grant gives 403; access stops when the grant expires; each operation is in the audit log | T2.13, T2.14, T2.29 | Claude | Sonnet | todo |
| T2.31 | Account deletion and data export request: deletion from the app, revocation of the Apple token, retention period and restore, anonymization, deletion or anonymization of GitHub Issues, full export (ZIP) | End-to-end: no data of the deleted account remains; the audit log is anonymous; the chain verification passes | T2.26, T2.30 | Claude | Sonnet | todo |
| T2.32 | Cluster: PostgreSQL, Redis, object store and fake services in kind (local overlay); migration Job; mTLS between services (ADR-0009); network policies (only the API connects to the engine; `/internal` only from the OIDC reverse proxy); oauth2-proxy; secrets; Terraform modules for Cloud SQL, Memorystore, Cloud Storage | `make kind-test` passes with the new checks (`/internal` from the public ingress gives 404); `terraform validate` is clean | T2.11, T2.15, T2.30 | Claude | Sonnet | todo |
| T2.36 | Visual architecture flow diagram (high level): clients (iPhone/iPad, web), edge, application services (api, worker, ingest), calculation engine, data (PostgreSQL, Redis, NATS, object store) and all external connections, each as a box with one arrow (Apple sign-in, Google sign-in, App Store, APNs, email, GitHub, measurement devices iSpindel/RAPT, web payment provider, observability/OTel, cloud); for each connection only the name and the direction, no details | A mermaid diagram in `belgeler/mimari-akis.md` and a published visual page (Artifact); agrees with plan › Platformlar ve mimari; each external connection shows one time | T2.32 | Claude | Sonnet | todo |
| T2.33 | End-to-end suite: scenarios on compose (full flow of a Home user: login → setup → recipe → batch → package → measurement → notification → support request; isolation of two workspaces); `make e2e` and a CI workflow; a short suite in kind | `make e2e` is green locally and in CI | T2.21–T2.32 | Claude | Sonnet | todo |
| F2.2 | Independent security review: identity, session, authorization, workspace isolation, signature of the brew day package, webhook signatures, support panel access, file upload, import | Report `gelistirme/raporlar/F2.2-guvenlik.md` | T2.33 | Claude | Fable | todo |
| T2.34 | Fixes for the F2.2 findings | All critical and important findings are closed or are in the decision list; the end-to-end suite is green | F2.2 | Claude | Sonnet (fix), Opus (review) | todo |
| T2.35 | Stage 2 gate check | The gate items are satisfied; deviations are in an ADR | T2.9, T2.34, T2.36 | Claude | Opus | todo |

## Stages 3–6

The stages are in plan › Geliştirme düzeni and plan › Yol haritası. Opus writes the task list of a stage when the previous stage closes.

| Stage | Content | Gate |
|---|---|---|
| 3 Web | Home MVP screens | Playwright, accessibility |
| 4 iOS | SwiftUI, brew day package, Live Activity | macOS CI is green; device test (Oğuz) |
| 5 Home MVP beta | Cloud setup, beta | Interviews and golden test are complete; independent security review |
| 6 Business | Business screens | Step by step |

## Tasks for Oğuz (parallel)

| Task | Description | Necessary for | Status |
|---|---|---|---|
| O1 | User interviews (`plan/gorusme.md`) | Stage 5 gate | todo |
| O2 | Golden test numbers from the BeerSmith screen (`formuller/ALTIN-TEST.md`) | Stage 5 gate | todo |
| O3 | Apple Developer account, macOS CI (GitHub Actions macOS or Xcode Cloud) | T0.9, stage 4 | active: macOS CI runs on GitHub macOS machines; the Apple Developer account and the device test are not done |
| O4 | Google Cloud organization and billing | Stage 5 | todo |
| O5 | Delete the 7 old branches on GitHub | – | todo |
| O6 | Select a web payment provider (subscriptions outside the App Store) | T2.28 | todo |
