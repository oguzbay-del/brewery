# F1 karar listesi

**Durum:** 14 maddenin hepsine karar verildi (F1.2). Uygulama: formüller ve test vektörleri.

Kaynak: `gelistirme/raporlar/F1-formul-dogrulama.md` (11 Kasım 2026). Aşağıdakiler yargı gerektirdiği için spesifikasyonlara ve test betiklerine **işlenmedi**; yalnızca Ö5'e "[doğrulanacak]" işareti ve model farkını anlatan bir not eklendi. Her madde için raporun önerisi son satırdadır. Karar verilince ilgili `formuller/*.md` ve test betiği birlikte güncellenir.

Rapor numaraları: K = kritik, Ö = önemli, S = küçük (raporun "Küçük bulgular" listesindeki sıra), D = "Doğrulanacak varsayılanlar" tablosu.

## 1. Ö2: İnvert şeker ve şurupların kuru madde varsayılanı
- **Sorun:** Spesifikasyon invert şekeri %100 kuru madde sayıyor; şurup ve bloklar su içerir, bu yüzden hesap yaklaşık %20 eksik doz verir. Alan eklendi, varsayılan **1.0 kaldı**.
- **Seçenekler:**
  1. Varsayılan 1.0 kalsın, değer yalnız malzeme veritabanından gelsin.
  2. Invert şeker için varsayılan kuru madde %80 olsun (kaba, ürüne göre değişir).
  3. Kuru madde alanı zorunlu olsun; boşsa ekran uyarı versin.
- **Raporun önerisi:** 3 (ya da 2). Tipik değer %70–85, kesin değer ürüne göre (raporun kendi belirsizliği: "emin değilim"). Ürün verisiyle doğrulanmalı.
- **Karar (Opus, 11 Kasım 2026):** 3 + veritabanı: kuru madde alanı her şeker ve şurup malzemesinde; değer malzeme veritabanından gelir, boşsa ekran uyarır ve hesap durmaz (1.0 ile ama "kuru madde bilinmiyor" notuyla).

## 2. Ö3 (sorun 1): Fıçı kafa boşluğunun hesap sıcaklığı
- **Sorun:** Kafa boşluğu CO₂'si kondisyon sıcaklığında (20 °C) hesaplanıyor; fıçı servis sıcaklığına soğuyunca bir kısmı biraya geçer ve bira hedefin üstüne çıkar (2b koşullarında 4 °C'de 2.49 vol, +0.09 vol).
- **Seçenekler:**
  1. Mevcut hâl: kondisyon sıcaklığında hesapla, bilinen sınır olarak not düş (şu anki durum).
  2. Kafa boşluğu dengesini servis sıcaklığında kur (toplam CO₂ korunumu).
  3. İki sıcaklığı da göster; kondisyon sıcaklığındaki tepe basıncı güvenlik için ayrıca ver.
- **Raporun önerisi:** 2 ya da 3. (Havalı fıçıda gösterge basıncına 1 atm eklenmesi tartışmasız olduğu için uygulandı: `toplam_gosterge_bar`.)
- **Karar (Opus, 11 Kasım 2026):** 2 + tepe basıncı: şeker, toplam CO₂ korunarak servis sıcaklığındaki denge hedef vol'ü verecek biçimde hesaplanır; kondisyon sıcaklığındaki tepe basıncı ayrıca gösterilir (fıçı güvenlik sınırıyla karşılaştırılır).

## 3. Ö5: BW mash pH eğimi ve varsayılan model
- **Sorun:** BW eğimi örtük tampon kapasitesini yaklaşık 18.4 mEq/(kg·pH) yapıyor; literatürde baz maltlar için kabaca 30–50, MPH ≈ 38. T3'te BW 1.77 mL, MPH 3.52 mL asit öneriyor (2 kat), BW RA'nın pH etkisini iki kat büyük gösteriyor. Raporun belirsizliği: sabitler BeerSmith'ten çıkarım, Bru'n Water'ın gerçek modeli doğrulanamadı.
- **Seçenekler:**
  1. BW'yi BeerSmith uyumu için birebir koru, varsayılan MPH olsun.
  2. Yalnız MPH sunulsun, BW "BeerSmith uyumluluğu" ekranında kalsın.
  3. İkisini de göster, aradaki farkı kullanıcıya açıkça yaz; ölçülmüş mash pH verisiyle kalibre et.
- **Raporun önerisi:** Ölçüm verisiyle karşılaştırılana kadar MPH varsayılan, BW "doğrulanacak". (03 §5.2'ye işaret ve not eklendi, varsayılan değişmedi.)
- **Karar (Opus, 11 Kasım 2026):** MPH varsayılan; BW seçenek ve "doğrulanacak". Kesin varsayılan altın testteki ölçülmüş mash pH ile (ALTIN-TEST 12 ve 13).

## 4. Ö6 (madde 2): Vergi girdisi yuvarlama kuralları
- **Sorun:** ABV ve °P'nin vergi için kaç haneye, hangi yönde yuvarlanacağı ülkeye göre değişir (İngiltere'de bir ondalığa aşağı kesme olduğu hatırlanıyor, doğrulanmadı). Motor artık bu kuralı veri tablosundan okuyor; varsayılan "yuvarlama yok".
- **Seçenekler:**
  1. Varsayılan yuvarlama yok; her ülke satırı kuralını açıkça yazsın.
  2. Varsayılan "bir ondalığa en yakın" olsun.
  3. Kural tanımsız satırda motor hata versin.
- **Raporun önerisi:** 1 + 3. Resmi kaynaktan (HMRC, BierStG, TTB) doğrulanmadan hiçbir ülke satırı "doğrulandı" sayılmamalı.
- **Karar (Opus, 11 Kasım 2026):** 1 + 3: varsayılan yuvarlama yok; kuralı tanımsız ülke satırında motor hata verir. Hiçbir ülke satırı resmi kaynakla doğrulanmadan "doğrulandı" sayılmaz.

## 5. Ö6 (madde 3): Son kademenin üstü
- **Sorun:** `kademeli_hacim`de son kademe aşılırsa kalan hacim sessizce vergisiz hesaplanıyordu. Uygulandı: son kademe `None` (açık uçlu) olabilir, aksi hâlde hata. Ürün kararı: gerçek tablolarda son kademe açık uçlu mu, yoksa aşım "kademe tablosu eksik" uyarısı mı olmalı?
- **Seçenekler:** (1) Tablolar açık uçlu girilsin, aşım hata olsun (şu anki hâl). (2) Aşımda son oran uygulansın. (3) Aşımda vergi hesaplansın, rapor "kademe aşıldı" uyarısı versin.
- **Raporun önerisi:** "açık uçlu olmalı ya da hata vermeli", yani 1.
- **Karar (Opus, 11 Kasım 2026):** 1: tablolar açık uçlu girilir; kapalı son kademe aşılırsa hata.

## 6. Ö7: No-chill 60 °C kesmesi ve aktivasyon sabiti
- **Sorun:** 60 °C altı katkı ihmal ediliyor; `Ftemp(60 °C) = 0.0435`. Kesme eşdeğer süreyi T₀ 95 °C için %13, T₀ 80 °C için %42 azaltıyor. Ayrıca Malowicki–Shellhammer sabitiyle (7.9·10¹¹·e^(−11858/T)) aynı örnek 25.3 dk, BeerSmith sabitiyle 32.9 dk veriyor (±%25 belirsizlik).
- **Seçenekler:**
  1. Kesmeyi 60 °C'de bırak, yalnız "doğrulanacak" notuyla belgele (şu anki durum).
  2. Kesmeyi kaldır ya da 40 °C'ye çek.
  3. Aktivasyon sabitini de yayımlanmış değerle değiştir.
- **Raporun önerisi:** 2. Model zaten varsayılan değil; ölçülmüş IBU ile kıyaslanmadan açılmaması kararı yerinde. (Test vektörleri 3a, 3d, 3e kesmeyle üretildi; karar çıkınca değişir.)
- **Karar (Opus, 11 Kasım 2026):** 2: kesme 40 °C'ye çekilir. Aktivasyon sabiti 02'deki IBU motoruyla tutarlı kalsın diye değişmez; iki sabitin farkı notta kalır. Model varsayılan değil (basit kural varsayılan).

## 7. D: Küp τ₂₀ varsayılanı
- **Sorun:** Kaba ısı dengesi 20 L için 4.4–6.6 saat veriyor; spesifikasyon 4 saat.
- **Seçenekler:** (1) 4 saat kalsın. (2) 5 saat olsun. (3) Kullanıcı ölçümü girene kadar model kapalı kalsın (zaten böyle).
- **Raporun önerisi:** 2 (5 saat daha savunulur), ölçümle doğrulanmalı. Vektör 3a/3b/3d/3e değişir.
- **Karar (Opus, 11 Kasım 2026):** 2: τ₂₀ = 5 saat, "doğrulanacak".

## 8. D: İsinglass dozu, cask kondisyon süresi, DME yığın yoğunluğu
- **Sorun:** İsinglass 2 mL/L raporun hatırladığı üretici aralığının (yaklaşık 1.7–7 mL/L) alt ucunda; cask kondisyonu 12 °C'de 3 gün alt uçta (pratik 3–7 gün); DME yığın yoğunluğu 0.60 g/mL için rapor emin değil.
- **Seçenekler:** (1) Değerler kalsın, üreticiden doğrulanana kadar "doğrulanacak". (2) İsinglass 3–4 mL/L'ye, kondisyon 5 güne çıkarılsın. (3) Alanlar boş gelsin, kullanıcı girsin.
- **Raporun önerisi:** 1; ürün verisi gerekli. Cask değerleri İngiltere'de bir iki üreticiyle ve Cask Marque yönergeleriyle kontrol edilmeli (11 §5 zaten yazıyor).
- **Karar (Opus, 11 Kasım 2026):** 1: değerler kalır, "doğrulanacak"; cask değerleri İngiltere'de üretici ve Cask Marque ile kontrol edilecek.

## 9. S1: CO₂ g/vol sabiti
- **Sorun:** 1 vol CO₂ için `11` 1.9635 (ideal gaz), `09` §3.1 1.977 (gerçek gaz), `04` §3.7 1.9642 ve 1.96042 kullanıyor; aynı Balling hesabı cask ve spunding'de %0.7 farklı sonuç veriyor (TV 3.1: 2.733 yerine 2.715 g/L).
- **Seçenekler:** (1) 1.9635 her yerde. (2) 1.977 her yerde. (3) BeerSmith uyumu için 04'te eski değerleri koru, yeni bölümlerde tek sabit.
- **Raporun önerisi:** Tek sabit seçilmeli; öneri açıkça yok (ideal gaz 1.9635 yeni bölümlerle tutarlı).
- **Karar (Opus, 11 Kasım 2026):** 1: yeni bölümlerde (09, 10, 11) tek sabit 1.9635 g/L/vol (ideal gaz, 22.414 L/mol). 04'teki BeerSmith değerleri yalnız "BeerSmith uyumu" notunda kalır; motor 1.9635 kullanır.

## 10. S2: Priming CO₂ verimi iki ayrı varsayım
- **Sorun:** Şeker için kuramsal verim (glukoz 0.4886, sakaroz 0.5143 g CO₂/g), kalan ekstrakt için Balling (0.4629); fark %5–10. Mevcut durum 11 §2.1'e yazıldı, seçim yapılmadı.
- **Seçenekler:** (1) Şeker kuramsal, ekstrakt Balling kalsın (şu anki hâl). (2) İkisi de Balling. (3) İkisi de kuramsal.
- **Raporun önerisi:** Seçim yazılmalı; belirli bir seçenek önerilmedi.
- **Karar (Opus, 11 Kasım 2026):** 1: şeker kuramsal (priming şekeri tamamen fermente olur, maya büyümesi ihmal edilir), kalan ekstrakt Balling (maya biyokütlesini içerir). Gerekçe 11 §2.1'de yazılı kalır.

## 11. S9: Az okumalı tahminde geç okumaların ağırlığı
- **Sorun:** Logaritmik uzayda orijinden geçen en küçük kareler, FG'ye en yakın okumaya aşırı ağırlık veriyor; bir puanlık okuma hatası ±22 saat kaydırıyor.
- **Seçenekler:** (1) Olduğu gibi, FG ± σ aralığı bunu örtsün. (2) `SG − FG < 0.003` olan okumaların ağırlığı düşürülsün. (3) Bu okumalar tahminden çıkarılsın.
- **Raporun önerisi:** 2 "düşürülebilir"; bu bir model değişikliği olduğu için işlenmedi.
- **Karar (Opus, 11 Kasım 2026):** 2: `SG − FG < 0.003` olan okumaların ağırlığı `(SG − FG)/0.003` ile doğrusal düşürülür (ağırlıklı en küçük kareler); test vektörleri güncellenir.

## 12. S12: 03 model ayrıntıları
- **Sorun:** (a) Grist fonksiyonu 14 SRM'de süreksiz (12.09 → 11.34). (b) Mash asidinde ayrışma kesri hedef pH yerine sabit 5.4'te; model 0 sparge hesabı laktik asidin %97 ayrışmasını hesaba katmıyor (%3 eksik). (c) Sparge hedefi mash hedefiyle aynı alan (5.4 için 2.86 mL, 6.0 için 2.15 mL). (d) Tebeşirin tam çözündüğü varsayımı alkaliniteyi büyük gösteriyor. Hepsi 03 §9'a not düşüldü.
- **Seçenekler:** (1) BeerSmith uyumu için hepsi birebir kalsın. (2) Kendi modelimizde düzeltilsin (sürekli grist, hedef pH'ta ayrışma, ayrı sparge hedefi). (3) Düzeltmeler "BeerSmith uyumlu" ve "düzeltilmiş" olarak iki mod sunulsun.
- **Raporun önerisi:** Belirgin öneri yok; (c) için ayrı sparge hedefi ürün açısından mantıklı.
- **Karar (Opus, 11 Kasım 2026):** 2 (a, b, c): kendi modelimizde süreklilik, hedef pH'ta ayrışma ve ayrı sparge hedefi. (d) tebeşir: model değişmez; tebeşir eklenince ekran "çözünürlük düşük, asitli mash olmadan etkisi abartılı olabilir" uyarısı verir.

## 13. S13: Etiket enerjisinde tahmini ABV yolu
- **Sorun:** Tahmini ABV 0.79661 ile türetilip gram'a 0.78924 ile çevriliyor (4.056 g); doğrudan `ABW × SG × 0.9982` 4.086 g (%0.75, ~1 kJ). `RE × SG`'de su yoğunluğu da yok (%0.2). Vektör 1.1–1.3 mevcut yola göre.
- **Seçenekler:** (1) Olduğu gibi (ölçülmüş ABV'de doğru). (2) Tahminde ABW'den doğrudan git. (3) Her iki yolu da göster, farkı not et.
- **Raporun önerisi:** 2 (tahmini değerde daha tutarlı). Seçilirse 1.1 ve 1.2 vektörleri değişir (kJ düzeyinde).
- **Karar (Opus, 11 Kasım 2026):** 2: tahmini ABV'de doğrudan ABW yolu; 1.1 ve 1.2 vektörleri güncellenir.

## 14. Ö1: "Tetik" eşiği aşağı yönü
- **Sorun:** Ölü bölge düzeltmesi yalnız yukarı yönde (`FG∞ > FG_tarif + Δ − 0.001`) uygulandı, çünkü raporun düzeltmesi yalnız onu verdi. Aşağı yönde eski sabit 0.004 farkı (`FG∞ < FG_tarif − 0.004` ise tahmin) korundu.
- **Seçenekler:** (1) Aşağı yön sabit 0.004 kalsın. (2) Aşağı yön de Δ'ya bağlansın. (3) Aşağı yönde tahmine hiç geçilmesin.
- **Raporun önerisi:** Belirtmedi.
- **Karar (Opus, 11 Kasım 2026):** 2: aşağı yön de Δ'ya bağlanır (simetrik).

## Uygulanan ama sizin bilmeniz gereken seçimler (karar değil, bilgi)
- **Karşılaştırma birimi:** rapor "1 ya da 0.1 puan" dedi; 0.1 puan birimi sentetik eğri vektörünü (1a: 74 → 73. saat) oynattığı için 0.01 puan (`round(SG × 100000)`) seçildi. Tolerans 1e-9.
- **Vektör 10/1f** hatayı gizliyordu: 1.020 → 1.019 (96. saat) bir puanlık durgunluktu ve eski kayan nokta hatası yüzünden "tutarsız" çıkıyordu. Doğru kuralla "takılmış olabilir" çıkar; vektör 46 saat arayla okumalarla (94. saat) yeniden kuruldu. Yeni 1g vektörü 1.014 → 1.013 çiftini kaydeder.
- **Lagering:** tavan (`ceil`) korundu, yalnız tam sayı puan üzerinden; OG 1.048 → 6, 1.072 → 9 hafta.
