# Durum

Her oturum önce bu dosyayı okur. En fazla bir sayfa. Her iş bitince güncellenir.

**Aşama:** 1 Hesap motoru (Aşama 0 kapandı: ADR-0002)
**Sıradaki iş:** T1.11 `WaterService` (profil, tuz, asit, mash pH), sonra Yeast, Fermentation, Brewery, T1.13 kapı.
**Yöntem:** ADR-0003 (Python → JSON vektör → Rust parite testi; `make vektor`).

## Ortam

- Kurulu: Rust 1.97, Node 22, Docker istemcisi. Oturum başında `dockerd` elle başlatılır.
- scratchpad/bin: terraform 1.17.2, kind 0.33.0, kubectl 1.37.2, actionlint 1.7.12. Yeni oturumda scratchpad boşsa yeniden indirilir (README'lerde yazılı).
- Bu ortamda imaj derlemesi: `make up BUILD_CA_FILE=/root/.ccr/ca-bundle.crt` (TLS vekili).
- CI: GitHub Actions, 6 iş akışı; iOS GitHub macOS makinelerinde.

## Engeller

- Yok. Açık kalanlar ADR-0002'de (Terraform hukuki/maliyet kararları, bulut hesabı O4, AppIcon).

## Son kararlar

- 11.11.2026: tam geliştirme; görüşme ve altın test Aşama 5 kapısı.
- ADR-0001 depo düzeni; ADR-0002 Aşama 0 kapanışı ve sapmalar.
- F1: formüller Fable ile bağımsız doğrulandı; 22 bulgu ve 14 karar işlendi. Kayan nokta ilkesi: gravite eşikleri tam sayı birimde (`round(SG*100000)`), sürekli büyüklükte 1e-9 (formuller/00-OKU-BENI.md).
- Her aşamaya yeni oturum.
- ADR-0003: motor paritesi; bölüm başına modül; motor fonksiyonları spesifikasyon birimini alır; para `rust_decimal`.

## Son biten işler (Aşama 1)

T1.0 ADR-0003 · T1.1–T1.10 portlar · T1.14 referanssız hesaplar · T1.12 Fable incelemesi (5 rapor) · T1.15 bulgu düzeltmeleri · T1.16 ortak model, hata tipi, adlar (ADR-0004) · T1.11 ToolService, RecipeService, MashService tamam (8 RPC; mash programı ve BIAB dahil, 41 uç testi, 1e-9 vektör eşitliği). 9970 parite vakası, hepsi eşit.

## Aşama 0 (kapandı)

T0.1 depo düzeni · T0.2 Rust iskeleti · T0.3 Docker Compose · T0.4 API sözleşmesi · T0.5 web · T0.6 iOS · T0.7 Kubernetes ve kind · T0.8 Terraform · T0.9 CI · T0.10 modül belgeleri · T0.11 kapı · T0.12 koyu yüksek karşıtlık · F1, F1.1, F1.2 formül doğrulaması
