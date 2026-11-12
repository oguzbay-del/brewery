# Bira Demleme Hesap Motoru — Formül Spesifikasyonu

Kaynak: BeerSmith 4.0.22 (Linux `.deb`) — hesap fonksiyonları Ghidra ile decompile edilip incelendi; Mac/Windows sürümleriyle aynı kaynak koddan derlenmiş (veri dosyaları birebir aynı). Formüller kendi ifademizle, uygulanabilir spesifikasyon olarak yazıldı.

## Dosyalar

| Dosya | İçerik |
|---|---|
| `01-yogunluk-alkol-renk.md` | Fermentable puanları, OG/FG, sanal OG, verim, mash sıcaklığına göre FG düzeltmesi, ABV/ABW (Balling), gerçek ekstrakt, attenuasyon, kalori/karbonhidrat, UK vergi ABV, renk (Morey), renk/gravite ayarlama, refraktometre, hidrometre düzeltmesi, birim dönüşümleri |
| `02-hop-ibu.md` | Tinseth, Rager, Garetz; tarif IBU motoru (geç ekstrakt SG segmentleri), whirlpool ve kaynatma sonrası katkı, rakım/kaynama sıcaklığı etkisi, form/kullanım düzeltmeleri, AAU, acılık ayarlama, hop oranları, hop yaşlanması |
| `03-su-ph.md` | Tuz–iyon katkı matrisi, mash/sparge/kaynatma/fermentör su profilleri, alkalinite, RA (Kolbach), SO₄/Cl, mash pH modeli (MPH ve BW), laktik/fosforik/asit malt hesapları, sparge asitleme, NNLS ile profil eşleştirme |
| `04-maya-mead-karbonasyon.md` | Maya canlılığı, pitch rate hedefleri, starter büyüme modeli (stir plate, 2 kademe), kuru maya, mead besinleri (TOSNA/TiOSNA, YAN), şarap sülfit/sorbat, tatlandırma, priming şekeri, fıçı basıncı |
| `05-mash-hacimler.md` | Strike/infüzyon/dekoksiyon ısı dengesi, BIAB, batch/fly sparge, tahıl emilimi ve hacmi, kaynatma kaybı, soğuma büzülmesi, tam hacim zinciri, parti-gyle, seyreltme |
| `06-eksik-hesaplar.md` | Spesifikasyonda henüz olmayan hesaplar: ekranların ihtiyaçları ve rakiplerle karşılaştırmadan çıkan liste, önceliğe göre |
| `07-ev-mvp-ek-hesaplar.md` | Ev MVP ek hesapları: fermantasyon tahmini, cihaz kalibrasyonu, ısınma ve soğuma süresi, ekipmana göre ölçekleme, iyon dengesi ve sertlik, hop ve meyve kayıpları (BeerSmith'ten değil, bizim tasarımımız) |
| `08-isletme-cekirdek-hesaplar.md` | İşletme çekirdek hesapları: maliyet (FIFO, su, enerji, ambalaj), harmanlama, maya tekrar kullanımı (hücre sayımı, slurry), profesyonel verim (brewhouse, dönüşüm, süzme), malt analizinden ekstrakt potansiyeli |
| `09-isletme-2-hesaplar.md` | İşletme 2. sürüm: AB etiket enerjisi, vergi motoru (yöntemler; oranlar veri tablosunda, örnek oran tablosu kaynaklı), spunding |
| `10-elle-giris-ekstrakt-nochill.md` | Az okumayla (elle) fermantasyon tahmini, ekstrakt araçları (LME/DME/bal dönüşümleri, kısmi kaynatma, üstüne su), soğutmasız (no-chill) IBU ayarı |
| `11-tetikleyici-fici-cask.md` | Fermantasyon adımına gravite tetikleyicisi (diacetyl rest, FG sabit, lagering), fıçıda priming (kafa boşluğu dahil), cask: hedef CO₂, kalan ekstrakt, priming, durultucu, kondisyon süresi |
| `testler/` | Her bölüm için Python referans uygulaması ve test vektörleri (`python3 testler/02_hop_ibu.py` gibi çalışır) |
| `ALTIN-TEST.md` | BeerSmith ekranı ve gerçek ölçümle karşılaştırılacak 16 senaryo. Boştur. `plan/gorusme.md` dolmadan doldurulmaz. `testler/` bunun yerine geçmez |
| `HESAP-MOTORU-TAMAMI.md` | 01–05 bölümlerinin tek dosyada birleşimi (06–11 ayrı dosyalarda) |

## Önemli genel notlar

- **Dahili birimler:** BeerSmith içeride İngiliz birimleri kullanıyor — hacim fl oz (`/128` = galon), ağırlık oz, sıcaklık °F, gravite SG, renk SRM. Kendi uygulamanızda metrik iç birim seçebilirsiniz; dönüşüm sabitleri dokümanlarda.
- **"Belirsiz" işaretli kısımlar:** Her bölümün sonunda listelendi. Bunlar ya koddan kesin çıkarılamadı ya da programda hata gibi görünen davranışlar (ör. bazı sabitlerin iki yönde farklı yazılması). Birebir uyum mu, doğru formül mü istediğinize göre karar verin.
- **Kayan nokta karşılaştırması (Rust motoru da uyar):** gravite (SG) eşik karşılaştırmaları ikili kayan nokta ile yapılmaz; değer önce tam sayı birime çevrilir, 1 birim = 0.00001 SG (0.01 puan), `u(SG) = round(SG × 100000)`, eşikler de aynı birimde yazılır (0.001 → 100, 0.004 → 400, 0.0005 → 50). Puan sayan hesaplar (ör. lagering haftası) önce tam sayı puana çevirir, yönü açık yuvarlar (`ceil` ya da `round`). Para ve vergi hesabı ondalık tiple yapılır (09 §2.2). Eğim, oran gibi sürekli büyüklüklerde açık tolerans (1e-9) kullanılır. Kapsam: bütün bölümlerdeki (01–11) gravite eşikleri (T1.12 kararı; önce yalnız 10 ve 11'deydi). Sürekli büyüklükte eşik (ör. ağırlıklı ortalama diastatik güç < 35 °L) `x < eşik − 1e-9` biçiminde yazılır. Stok miktarı ve para ondalık tiple tutulur (08 ve 09). Motor ara değeri gösterim için yuvarlamaz (ADR-0004 madde 7). Gerekçe: `1.014 − 1.013 = 0.001000000000000112` olduğundan `≤ 0.001` yanlış çıkar; `(1.048 − 1) × 1000 = 48.00000000000004` olduğundan `ceil(/8)` 6 yerine 7 verir.
- **Test vektörleri** formüllerin Python uygulamasından hesaplandı; BeerSmith'in kendi ekran çıktılarıyla karşılaştırılmadı. Kendi uygulamanızı yazmadan önce BeerSmith'te birkaç tarifi açıp sonuçları karşılaştırmanız önerilir.
- Sistem tasarımında not edilen yaklaşım (önceki analizden): alan/ayar tanımları veri tablosunda, her kayıtta UUID, güncelleme zamanı ve sürüm numarası (çevrimdışı senkron yok; plan › Veri modeli), raporlar şablonla.

## Hata kuralı (Karar T1.16, ADR-0004 madde 3 ve 5)

Referans istisna attığında ya da sonuç NaN/sonsuz olacaksa motor `Err(engine::Error)` döndürür; vektörde beklenen `{"hata": "ValueError"}` (Python istisna sınıfı `ValueError` olarak normalleştirilir, Rust testi yalnız `hata` anahtarını denetler). Sessiz sentinel değerler kalktı: bilinmeyen kod (birim, asit, denklem, form, ajan, azot faktörü, yöntem) hata verir; ulaşılamayan hedef (`tinseth_weight`, `rager_weight`, `garetz_weight`, `blend_volume_for_target`, `hop_ibu_adjust`) hata verir; Garetz yinelemesi sınırda yakınsamazsa hata verir; mash asit miktarı yalnız 0–3 asit kodlarında tanımlıdır; para yuvarlama hanesi 0–12. Referansta bu davranış `Karar (T1.16)` yorumlarıyla işaretlidir. Sonlu olmayan girdi vektöre yazılamadığı için Rust birim testleriyle sınanır.

## Adlandırma, imza ve gösterim (Karar T1.16 C, ADR-0004 madde 4, 6, 7)

- **Birim ekleri:** açık işlev parametreleri ve `pub` yapı alanları birim ekini taşır (`_oz`, `_floz`, `_gal`, `_l`, `_kg`, `_g`, `_f`, `_c`, `_min`, `_h`, `_pct`, `_ppm`, `_p` (°Plato), `_srm`, `_kj`, `_billion` ...); oran, SG ve birimi adında olan nicelikler (`ibu`, `brix`, `kcal`) eksiz kalabilir. Vektör anahtarları değişmez; Rust testleri eşler.
- **Girdi yapısı:** dörtten fazla girdisi olan işlev adlı alanlı bir girdi yapısı alır (`hops::HopIbuInput`, `mash::StrikeTempInput`, `water::AdjustedPhInput` ...); birden fazla çıktısı olan işlev adlı alanlı yapı döndürür (demet yok).
- **Tek hata yüzeyi:** `TaxError`, `ManualError`, `TriggerError` kalktı; hepsi `engine::Error` (`InvalidInput`, `NoConvergence`, `Unreachable`, `EmptyInput`, `DivisionByZero`, `Overflow`, `InvalidBands`). Vektördeki Python istisna adı (`ValueError`, `IndexError`, `ZeroDivisionError`, `OverflowError`) Rust testinde varyanttan türetilir.
- **Gösterim yok:** motorda metin etiketi ve gösterim yuvarlaması yoktur (`sparge_plan`, `enum_name`, `pitch_range`, `sulfate_chloride_label`, `row_status`, `Warning::message`); referansta bunlar gösterim yardımcısı olarak kalır. Kaynak: `sparge_plan` vektörü artık yuvarlanmamış hacim ve eylem kodu içerir.
- **Tek uygulama:** `plato_to_sg` (`units::to_sg`), `lactic_acid_ra` (`water::misc_acid_ra`, asit = laktik), `lactic_ml` (`water::sparge_acid_ml`, laktik), `wn_grain_absorb` (`mash::grain_absorb`, BIAB yok), `scale_recipe` (`home::scale_factor`) motordan kaldırıldı; vektör vakaları Rust testinde tek uygulamaya yönlendirilir.
