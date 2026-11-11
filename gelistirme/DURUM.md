# Durum

Her oturum önce bu dosyayı okur. En fazla bir sayfa. Her iş bitince güncellenir.

**Aşama:** 0 Temel
**Sürüyor:** T0.3 Compose, T0.4 API sözleşmesi, T0.8 Terraform (Sonnet), F1 formül doğrulaması (Fable)
**Sıradaki iş:** T0.3 Docker Compose (Sonnet) · T0.4 API sözleşmesi iskeleti (Sonnet); ikisi de T0.2'ye bağlıydı, artık başlayabilir
**Paralel başlatılabilir:** F1 formül doğrulaması (Fable)

## Ortam

- Kurulu: Rust 1.97, Node 22, npm 10, Docker istemcisi.
- Eksik: Docker arka plan servisi çalışmıyor; kind, kubectl, Terraform, protoc kurulu değil (indirme adresleri erişilebilir). T0.3 başında `dockerd` başlatılmaya ve araçlar kurulmaya çalışılır; olmazsa engel olarak yazılır.

## Engeller

- T0.9: GitHub iş akışı dosyalarının bu ortamdan gönderilmesi engellenebilir (dal silme engellenmişti). Engellenirse dosyalar hazırlanır, gönderimi Oğuz yapar.

- iOS ilk macOS derlemesinde olası sorunlar (T0.6): Swift 6 katı eşzamanlılık, test barındırma (test host), Info.plist ayar adları, simülatör adı. Derleme O3 (macOS CI) ile.

- Tasarım sistemi: koyu tema için yüksek karşıtlık değerleri yok (tokens.json yalnız açık tema için 5 değer veriyor). Web ve iOS şimdilik koyu + karşıtlıkta koyu değerleri kullanıyor. Tasarım sistemine eklenecek.

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
