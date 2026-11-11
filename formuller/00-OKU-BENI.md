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
- **Test vektörleri** formüllerin Python uygulamasından hesaplandı; BeerSmith'in kendi ekran çıktılarıyla karşılaştırılmadı. Kendi uygulamanızı yazmadan önce BeerSmith'te birkaç tarifi açıp sonuçları karşılaştırmanız önerilir.
- Sistem tasarımında not edilen yaklaşım (önceki analizden): alan/ayar tanımları veri tablosunda, her kayıtta UUID, güncelleme zamanı ve sürüm numarası (çevrimdışı senkron yok; plan › Veri modeli), raporlar şablonla.
