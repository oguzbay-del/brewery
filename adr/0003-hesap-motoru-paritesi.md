# ADR-0003: Hesap motoru ve Python paritesi

- **Durum:** kabul edildi
- **Tarih:** 11.11.2026
- **İş:** T1.0

## Bağlam

Aşama 1 kapısı: Rust motoru Python referansıyla (`formuller/testler/*.py`) bütün test vektörlerinde eşit. Referans betikler sonuçları yalnız ekrana basıyor, karşılaştırılabilir veri yok.

## Karar

1. **Vektör dosyaları.** Her bölüm için `formuller/testler/parite/NN_*.py` referans betiği modül olarak yükler (çıktısı bastırılır), fonksiyonları seçilmiş girdilerle çağırır ve `formuller/testler/vektorler/NN.json` yazar. Vaka biçimi: `{"fonk": "<rust_anahtari>", "girdi": {...}, "beklenen": ...}`. Ondalık değer `{"ondalik": "12.34"}` olarak yazılır. Referans betiklerdeki mevcut fonksiyonlar değişmez; değişmesi gerekirse önce formül spesifikasyonu değişir. Spesifikasyonda olup referansta olmayan hesap için önce referansa fonksiyon eklenir (T1.14), Rust ondan sonra yazılır.
2. **Üretim ve denetim.** `make vektor` (`python3 -I formuller/testler/parite/uret.py`) hepsini yeniden üretir. CI (belgeler iş akışı) üretip `git diff --exit-code` ile vektörlerin güncel olduğunu denetler; sunucu iş akışı vektörler değişince de çalışır.
3. **Rust testi.** `sunucu/crates/engine/tests/parite_NN.rs` dosyayı `tests/ortak` ile okur; her `fonk` bir motor çağrısına eşlenir, eşlenmeyen vaka testi düşürür. Tolerans: sayılarda `|a − b| ≤ 1e-12 + 1e-11 · |b|` (T1.12 P2); JSON sayıları en yakın `f64`'e okunur (`serde_json` `float_roundtrip`, T1.12 P1), tam sayı, metin, mantıksal ve ondalıkta tam eşitlik.
4. **Modül düzeni.** `engine` içinde bölüm başına bir modül: `units` (ortak dönüşümler), `density` 01, `hops` 02, `water` 03, `yeast` 04, `mash` 05, `home` 07, `brewery` 08, `brewery_tax` 09, `manual` 10, `trigger` 11. Bir bölümün başka bölümden ihtiyacı olan fonksiyon kopyalanmaz, o modülden çağrılır.
5. **Birimler.** Motor fonksiyonları spesifikasyondaki birimi alır (01–05 BeerSmith İngiliz iç birimleri, 07–11 metrik); dönüşüm API katmanının işidir. Her fonksiyonun belgesinde birim yazılır.
6. **Yuvarlama ve eşik.** Gravite eşikleri `units::gravity_units` (yarımda çifte, Python `round` ile aynı) ile tam sayı birimde karşılaştırılır. Python'un yuvarladığı yerde (ara adım ya da referansın döndürdüğü değer) `units::round_py` (Python `round` ile aynı: ikili değerin tam ondalık açılımı, yarımda çifte); başka yerde çıktı yuvarlanmaz.
7. **Bağımlılık.** `engine` IO'suz kalır. Saf hesap crate'i (para ve vergi için `rust_decimal`) kabul edilir; test bağımlılığı yalnız `serde_json`.
8. **NNLS (03 §NNLS).** Referans scipy kullanıyor ve CI'da scipy yok; parite betiği saf Python Lawson–Hanson yazar, scipy varsa onunla da karşılaştırır.

## Sonuçlar

- Bölüm işleri birbirinden bağımsız dosyalara dokunur, paralel yürür; yalnız 10 (07'ye) ve 11 (04, 07'ye) bağlı.
- Python referansı doğruluğun tek kaynağı olarak kalır; BeerSmith ekran karşılaştırması Aşama 5 altın testindedir.
