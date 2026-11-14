# ADR-0006: Aşama 1 kapanışı (hesap motoru)

**Durum:** kabul · **Tarih:** 13 Kasım 2026 · **İş:** T1.13

## Kapı

| Kapı maddesi | Sonuç |
|---|---|
| Python referansıyla Rust motoru bütün test vektörlerinde eşit | `make vektor` farksız; 10 bölümde 10 269 vaka, `cargo test -p engine` parite testlerinin hepsi eşit; işlenmeyen `fonk` hata verir (`ortak::run`), her vektör dosyasının testi var (`parite_00`) |
| Fable formül doğrulamasının bulguları kapandı | F1 (22 bulgu, 14 karar) F1.1–F1.2'de; T1.12 (5 rapor) T1.15, T1.16 ve T1.17'de kapandı. T1.16'da açık kalan üç madde (03 tipsiz grist, 07 C10 "bitti", 08 §4.3 süzme verimi) T1.17'de spesifikasyon → referans → vektör → Rust → servis sırasıyla kapandı |
| Motor gRPC ile sunuluyor (T1.11) | 8 servis (Engine, Tool, Recipe, Mash, Water, Yeast, Fermentation, Brewery), ADR-0005; her RPC'nin vektörden türetilmiş uç testi; TCP üzerinden uçtan uca test (`tests/uc_uca.rs`); ikili gerçek portta açılıp kapanıyor |
| Birleştirme kapısı | `cargo fmt`, `clippy -D warnings`, `cargo test --workspace` (205 test) yeşil; sözleşme fark denetimi temiz; `araclar/tutarlilik.py` temiz |

## Sapmalar ve kararlar

- **Proto kapsamı ADR-0005 listesinden geniş:** Yeast'e şarap stabilizatörü ve tatlandırma, Fermentation'a fıçı/cask priming, no-chill IBU, Tilt dönüşümü, Brewery'ye maya tekrar kullanımı, malt analizi, diastatik güç denetimi eklendi (ADR-0005 madde 2 güncellendi).
- **Elle tetik günü (11 §1.3):** `few_forecast`'ın 4 haneye yuvarlanmış `k` değerini kullanır (10 §1 çıktısı zaten yuvarlı). Sonuç gün çözünürlüğünde olduğundan etkisi ihmal edilebilir; ADR-0004 madde 7'den bilinçli sapma.
- **psi↔bar:** tek kesin sabit `engine::units::PSI_BAR` (0.0689475729316836); 09 §3.3 güncellendi, vektörler değişmedi (2 hane yuvarlama).
- **Vergi raporu:** `tax_line_detail` LPA ve uygulanan oranı döndürür; `saf_alkol`'de oran indirim sonrası nettir.

## Açık kalanlar (Aşama 1'i kapatmaya engel değil)

Ayrıntı `gelistirme/raporlar/T1.11-motor-eksikleri.md`.

- 09 yöntem zinciri (bant + kademe + indirim) ve `ytd_hl` taşınması: veri tablosu tasarımıyla Aşama 2'de (T2.8 ile kapandı: 09 §2.2a).
- Elektrikli kaynatmada güç × süre enerji hesabı: istekte `additional_kwh`; ihtiyaç İşletme ekranında netleşir (Aşama 6).
- 07 C10'un ikinci yarısı (ızgara kenarına yapışan aralığın açık uçlu gösterimi), Tilt aralık uyarısı ve §2.2 sıcaklık düzeltmesi.
- Sparge hedef pH varsayılanı (6.0, doğrulanacak); 4–8 kodlu asitlerde mash miktarı tanımsız.
- Mead bal yüzdesinin sunucuda hesaplanması (fermentable başına OG puanı gerekiyor), `est_cells_ml` tanımı.
- 07 C14 °fH böleni (10.009, doğrulanacak), 07 C15 sıfıra yuvarlanan malzeme (Aşama 3 ölçekleme ekranı).
