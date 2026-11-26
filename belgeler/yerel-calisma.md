# Yerel geliştirme

Bu kılavuz depo kök dizininden çalışmayı anlatır. Yerel Compose ortamı geliştirme ve test içindir; gerçek sağlayıcılar yerine sahte OIDC, GitHub ve APNs servisleri kullanır. Üretim kimlik bilgilerini yerel örnek değerlerle karıştırmayın.

## Gereksinimler

| Araç | Sürüm / not |
|---|---|
| Docker Engine ve Docker Compose eklentisi | Compose dosyasını çalıştırabilecek kurulum gerekir; sabit sürüm belirtilmemiştir (to be verified). |
| GNU Make | Kök Makefile giriş noktasıdır; sabit sürüm belirtilmemiştir (to be verified). |
| Rust ve Cargo | sunucu/rust-toolchain.toml Rust **1.97.0** ve rustfmt, clippy bileşenlerini sabitler. |
| Node.js ve npm | Web için Node **22** (web/.nvmrc); web/package.json Node >=22 <23 ister. |
| Python | Üretim betikleri Python 3 kullanır. Belgeler CI iş akışı Python **3.13** kullanır. |
| Kubernetes araçları | kind ve kubectl PATH üzerinde olmalıdır. make kind-up Docker da kullanır. |
| Go | Sözleşme CI iş akışı oasdiff kurmak için Go stable kullanır; yerel sürüm ayrıca sabitlenmemiştir (to be verified). |

## Compose ortamını başlatma ve durdurma

Depo kökünde:

    make up
    make ps
    make smoke
    make down

- make up, altyapi/compose/compose.yaml dosyasını derler, servisleri arka planda başlatır ve sağlık denetimlerini en fazla 600 saniye bekler.
- make smoke servis sağlıklarını kontrol eder. Önce make up çalıştırılmalıdır.
- make logs günlükleri izler; tek servis için make logs S=api kullanılabilir. Göç servisi için make logs S=migrate kullanılabilir.
- make down servisleri kaldırır, kalıcı veri hacimlerini korur. make down ARGS=-v veri hacimlerini de siler ve yerel veriyi kaybettirir.
- make seed örnek kütüphane verisini (malzeme ve stiller; her satır "SAMPLE DATA" işaretlidir) yazar. Yalnız geliştirme içindir, önce make up çalıştırılmalıdır. İki kez çalıştırmak güvenlidir. make up bunu çalıştırmaz (compose profili seed).
- TLS'i proxy arkasında derlemek için make up BUILD_CA_FILE=/yol/ca.crt kullanılabilir.

### Yerel servis adresleri

Compose portları yalnızca 127.0.0.1 üzerinde yayımlanır. Değerler ortam değişkenleriyle değiştirilebilir.

| Servis | Yerel adres / port | Kullanım |
|---|---|---|
| API | http://127.0.0.1:8080 | HTTP API; /healthz sağlık yolu |
| Worker | 127.0.0.1:8081 | Worker portu |
| Ingest | 127.0.0.1:8082 | Ingest portu |
| Engine | 127.0.0.1:50051 | gRPC |
| PostgreSQL | 127.0.0.1:5432 | Yerel veritabanı |
| Redis | 127.0.0.1:6379 | Önbellek/servis altyapısı |
| NATS | 127.0.0.1:4222 | JetStream mesajlaşma |
| NATS monitor | http://127.0.0.1:8222 | NATS sağlık/izleme |
| Mailpit SMTP | 127.0.0.1:1025 | Giden test e-postaları |
| Mailpit arayüzü | http://127.0.0.1:8025 | Test e-postalarını görüntüleme |
| Fake GitHub | http://127.0.0.1:8090 | GitHub entegrasyon mock'u |
| Fake APNs | http://127.0.0.1:8091 | Apple Push mock'u |
| Fake OIDC | http://127.0.0.1:9000 | Sahte Apple ve Google OIDC |
| S3 uyumlu nesne deposu | http://127.0.0.1:8333 | SeaweedFS S3 API |

### Fake OIDC ve test kullanıcısı

Fake OIDC yalnız yerel geliştirme ve test içindir; gerçek Apple/Google hesabı veya üretim kimlik doğrulaması sağlamaz.

- Apple issuer: http://fake-oidc:9000/apple; JWKS: http://fake-oidc:9000/apple/jwks.
- Google issuer: http://fake-oidc:9000/google; JWKS: http://fake-oidc:9000/google/jwks.
- Token üretme uçları: POST http://127.0.0.1:9000/apple/mint ve POST http://127.0.0.1:9000/google/mint.

Fake OIDC varsayılan MintSpec konusu (sub) fake-user-1, audience değeri fake-client olur; e-posta ve ad varsayılan olarak boştur. Compose API yapılandırmasında kabul edilen audience değerleri Apple için com.demleme.app.dev, Google için demleme-dev.apps.googleusercontent.example olduğundan, API ile kullanılacak token bu audience değerlerinden uygun olanını taşımalıdır. Kaynakta hazır parola veya sabit test hesabı tanımlı değildir; token kimliği sub alanıdır. Fake OIDC herkesin token üretebildiği bir servistir; dış ağa açmayın.

## Testleri çalıştırma

### Sunucu (Rust)

Önce Compose servislerini ve PostgreSQL'i başlatın:

    make up
    make test

make test, sunucu/ içinde cargo test --workspace çalıştırır. CI'daki sunucu işinin temel yerel kontrolleri:

    cd sunucu
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    cargo run -p db --bin migrate
    cargo sqlx prepare --workspace --check -- --all-targets

SQLx kontrolü için göç uygulanmış PostgreSQL gerekir. Kök Makefile'daki make migrate yerel Compose veritabanına göçleri uygular.

### Web

    cd web
    npm ci
    npm run lint
    npm run format:check
    npm run typecheck
    npm run check:tokens
    npm test
    npm run build
    npm run e2e

Playwright Chromium tarayıcısı önceden kurulu olmalıdır. Depo kuralı yerel E2E için /opt/pw-browsers/chromium yolunu veya PW_CHROMIUM_PATH değişkenini kullanır. Bu projede playwright install komutunu çalıştırmayın; CI kendi tarayıcısını kurar.

### Compose uçtan uca testleri

    make up
    make e2e
    make down

make e2e, Compose ağına bağlı ayrı bir ana makine portunda (varsayılan 18081) geçici keyed API başlatır ve sunucu/crates/e2e testlerini çalıştırır. Test tamamlanınca geçici API konteynerini kaldırır.

### Kubernetes / kind

    make kind-up
    make kind-test
    make kind-down

make kind-up yerel kind kümesini kurar, Calico'yu ve imajları yükler, altyapi/k8s/overlays/yerel tanımlarını uygular. make kind-test; pod'ların hazır olmasını, API /healthz yanıtını, engine gRPC sağlığını ve ağ politikası denetimlerini kontrol eder. make kind-down kümeyi siler. TLS proxy arkasında make kind-up BUILD_CA_FILE=/yol/ca.crt; imaj derlemesini atlamak için SKIP_BUILD=1 kullanılabilir.

### Sözleşme ve istemci üretimi

    bash sozlesme/scripts/gen.sh

Bu komut OpenAPI belgesini, TypeScript istemci tiplerini ve katalog çıktısını üretir. Betik Rust API aracını çalıştırır; TypeScript üreticisi için gerekli npm bağımlılıkları yoksa npm ci çalıştırır; ardından Python katalog üreticisini çağırır. Yalnız doğrulama yapmak için:

    bash sozlesme/scripts/check.sh

## CI iş akışı ve yerel karşılığı

| İş akışı | CI neyi kontrol eder? | Yerel karşılık |
|---|---|---|
| sunucu.yml | Rust biçim, Clippy, workspace testleri, göç ve SQLx sorgu önbelleği | Yukarıdaki Sunucu komutları; önce make up |
| web.yml | ESLint, Prettier, TypeScript, token güncelliği, Vitest, build ve Playwright E2E | Yukarıdaki Web komutları |
| altyapi.yml – Compose | Compose yapılandırması, make up, smoke ve E2E | docker compose -f altyapi/compose/compose.yaml config --quiet; ardından make up, make smoke, make e2e |
| altyapi.yml – Kubernetes | Kustomize çıktıları ve kind smoke testleri | kubectl kustomize altyapi/k8s/base; overlay'ler için kubectl kustomize altyapi/k8s/overlays/<ad>; ardından kind komutları |
| altyapi.yml – Terraform | Terraform biçimi ve üç ortamda doğrulama | cd altyapi/terraform && terraform fmt -check -recursive; her envs/*/ dizininde terraform init -backend=false -input=false ve terraform validate |
| sozlesme.yml | OpenAPI ve TypeScript tiplerinin güncelliği / geriye uyumluluk | bash sozlesme/scripts/check.sh |
| belgeler.yml | Plan/konsept tutarlılığı ve formül test vektörleri | python3 -I araclar/tutarlilik.py; formuller içinde python3 -I testler/<dosya>.py; parite için python3 -I formuller/testler/parite/uret.py ve git diff --exit-code -- formuller/testler/vektorler |
| actionlint.yml | GitHub Actions YAML ve ifade kuralları | docker run --rm -v "$PWD:/repo" -w /repo rhysd/actionlint:1.7.12 -color |
| marketing-site.yml | Astro/TypeScript kontrolü, site build/route doğrulaması, tarayıcı QA ve Lighthouse | cd marketing-site && npm install && npm run check && npm run build && npm run validate:site && npm run test:browser; Lighthouse adımı CI betiğinde tanımlıdır |
| marketing-site-pages.yml | Ana dalda statik site build ve Pages dağıtımı | cd marketing-site && npm install && npm run check && npm run build && npm run validate:site; dağıtım yalnız CI'da çalışır |
| ios.yml | Token/proje doğrulaması, XcodeGen, SwiftLint ve iOS simülatör testleri | macOS ve Xcode gerekir; CI adımları ios/scripts/validate.py, xcodegen generate, swiftlint lint --strict ve xcodebuild test |
| yetki.yml | PR'daki değişen yolların korumalı yol listesine karşı kontrolü | GitHub PR bağlamı ve base ref gerekir; yerel tek-komut karşılığı tanımlanmamıştır |

**Belgeler CI notu (25 Kasım 2026):** Ana daldaki `.github/workflows/belgeler.yml` yol filtresi henüz `belgeler/**` içermiyor. PR #321 bu klasörü tetikleyici kapsamına ekler ve `python3 -I belgeler/uret-api-ozet.py --check` denetimini çalıştırır. PR #321 Claude incelemesinden geçip ana dala alındığında bu kılavuzdaki değişiklikler de ilgili iş akışını başlatır.

## Sorun giderme

- Bir servis hazır değilse make ps ile durumuna bakın; make logs S=<servis> ile günlükleri izleyin. Compose başlangıcında API ve diğer Rust servisleri PostgreSQL göçünün tamamlanmasını bekler; göç sorunu için make logs S=migrate çıktısını inceleyin.
- Bir port başka süreç tarafından kullanılıyorsa Compose portu yayımlayamaz. Port değerlerini altyapi/compose/compose.yaml içindeki ilgili *_PORT ortam değişkeniyle değiştirin.
- Docker Hub 429 geçici hız sınırı olabilir. kind-up.sh üç çekme denemesi yapar; üçü de başarısızsa imajlar yerelde mevcutsa uyarı verip mevcut imajlarla devam eder. İmajları yeniden derlemeyi/çekmeyi tamamen atlamak için SKIP_BUILD=1 kullanılabilir.
