# Durum

Her oturum önce bu dosyayı okur. En fazla bir sayfa. Her iş bitince güncellenir.

**Aşama:** 0 Temel
**Sürüyor:** F1.1 formül bulgularının işlenmesi (Sonnet)
**Sıradaki iş:** T0.7 Kubernetes ve kind (Sonnet) · T0.9 CI (Sonnet); ardından T0.11 kapı kontrolü (Opus)
**Paralel başlatılabilir:** F1 formül doğrulaması (Fable)

## Ortam

- Kurulu: Rust 1.97, Node 22, npm 10, Docker istemcisi.
- Docker: oturum başında `dockerd` elle başlatılır (günlük scratchpad'de). Bu ortamda imaj derlemesi için `make up BUILD_CA_FILE=/root/.ccr/ca-bundle.crt` gerekir (TLS vekili). Terraform 1.17.2 scratchpad/bin'de; kind ve kubectl kurulu değil (T0.7'de kurulur).

## Engeller

- T0.9: GitHub iş akışı dosyalarının bu ortamdan gönderilmesi engellenebilir (dal silme engellenmişti). Engellenirse dosyalar hazırlanır, gönderimi Oğuz yapar.

- iOS ilk macOS derlemesinde olası sorunlar (T0.6): Swift 6 katı eşzamanlılık, test barındırma (test host), Info.plist ayar adları, simülatör adı. Derleme O3 (macOS CI) ile.


## Son kararlar

- 11 Kasım 2026: tam geliştirmeye geçildi. Görüşme ve altın test kodun başlamasının değil Aşama 5'in (beta) kapısı.
- ADR-0001: depo düzeni, kod İngilizce ve belgeler Türkçe, Kustomize, XcodeGen, `is/T…` dalları, commit mesajı iş numarasıyla başlar.
- Her aşamaya yeni oturum açılır (token tasarrufu).

## Son biten işler

- T0.1 Depo düzeni ve ADR-0001 (11.11.2026)
- T0.2 Rust çalışma alanı: 6 crate, sağlık uçları, gRPC (protox, sistem protoc gerekmez), 9 test (11.11.2026)
- T0.6 iOS iskeleti: XcodeGen projesi, 27 renk açık/koyu/yüksek karşıtlık, doğrulama betikleri (11.11.2026; Swift derlemesi macOS CI'da)
- T0.5 Web iskeleti: Vite, React 19, TypeScript strict, token'lardan CSS, Vitest ve Playwright (400 ve 1280 px) (11.11.2026)
- T0.4 API sözleşmesi: utoipa ile OpenAPI koddan, `/v1/version`, ortak hata şeması, TypeScript tipleri, fark denetimi (11.11.2026)
- Düzeltme: kök `.gitignore`'daki `src/` her düzeydeki src klasörlerini yok sayıyordu; T0.2 ve T0.5 kaynak kodu depoya girmemişti. Kural `/src/` yapıldı, kaynaklar eklendi (11.11.2026)
- T0.8 Terraform iskeleti: 7 GCP modülü, dev, test, canlı; validate geçiyor. Açık kararlar: denetim kovasında WORM ve saklama süresi, CMEK kapsamı, yedeğin başka bölgeye kopyası (veri yeri), CI için Workload Identity (11.11.2026)
- T0.10 Modül CLAUDE.md dosyaları (11.11.2026)
- T0.12 Koyu tema yüksek karşıtlık token'ları; tasarım sistemi yayını güncellendi (11.11.2026)
- T0.3 Docker Compose: `make up` 10 servis sağlıklı (PostgreSQL, Redis, NATS JetStream, API, hesap motoru, worker, ingest, Mailpit, sahte GitHub ve APNs), `make smoke` geçiyor (11.11.2026)
- F1 Formül doğrulaması (Fable): 22 bulgu, 2 kritik (11.11.2026)
