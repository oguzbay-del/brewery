# Durum

Her oturum önce bu dosyayı okur. En fazla bir sayfa. Her iş bitince güncellenir.

**Aşama:** 0 Temel
**Sıradaki iş:** T0.2 Rust çalışma alanı (Sonnet)
**Ardından:** T0.5 Web iskeleti (Sonnet) · T0.8 Terraform iskeleti (Sonnet) · T0.6 iOS iskeleti (Sonnet); bu üçü T0.2 ile paralel yürüyebilir
**Paralel başlatılabilir:** F1 formül doğrulaması (Fable)

## Ortam

- Kurulu: Rust 1.97, Node 22, npm 10, Docker istemcisi.
- Eksik: Docker arka plan servisi çalışmıyor; kind, kubectl, Terraform, protoc kurulu değil (indirme adresleri erişilebilir). T0.3 başında `dockerd` başlatılmaya ve araçlar kurulmaya çalışılır; olmazsa engel olarak yazılır.

## Engeller

- T0.9: GitHub iş akışı dosyalarının bu ortamdan gönderilmesi engellenebilir (dal silme engellenmişti). Engellenirse dosyalar hazırlanır, gönderimi Oğuz yapar.

## Son kararlar

- 11 Kasım 2026: tam geliştirmeye geçildi. Görüşme ve altın test kodun başlamasının değil Aşama 5'in (beta) kapısı.
- ADR-0001: depo düzeni, kod İngilizce ve belgeler Türkçe, Kustomize, XcodeGen, `is/T…` dalları, commit mesajı iş numarasıyla başlar.
- Her aşamaya yeni oturum açılır (token tasarrufu).

## Son biten işler

- T0.1 Depo düzeni ve ADR-0001 (11.11.2026)
