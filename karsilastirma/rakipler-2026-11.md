# Rakip incelemesi: Brewfather, BeerSmith, Brew Cloud, Brewtarget, Brewer's Friend, Brew Meister, FoamByte

**İnceleme tarihi:** 20 Kasım 2026
**Kapsam:** İlk sürüm hedefi iPhone/iPad ve web; Ev ve İşletme modu; ABD ve İngiltere pazarı. ERP, topluluk ve herkese açık tarif paylaşımı, Android, ayrı Mac/Windows uygulaması ve lisans konusu bu raporun dışında; bunlar için öneri yazılmadı.
**Yöntem:** Resmî sayfalar ve App Store sayfaları WebFetch ile açıldı. FoamByte için mevcut dört rapor (`foambyte-*.md`) kullanıldı, yeniden araştırılmadı. BeerSmith 4 ve Brewtarget için `06-ozellik-veri-modeli-karsilastirma.md` kullanıldı. Plan için yalnız görevde belirtilen bölümler okundu.

**Okuma notu (kanıt düzeyi):**
- Sayfa özetleri küçük bir model tarafından çıkarıldı; rakamlar (fiyat, limit) yayına almadan önce sayfadan elle kontrol edilmeli.
- Bazı sitelerde (brewersfriend.com, homebrewtalk.com, Brewfather fiyat sayfası, Brewfather `/features`) sayfa açılamadı (403/404). Bu kaynaklar arama özetine dayanır ve "arama özeti" diye işaretlendi; arama özeti kaynak sayılmaz, bu yüzden bu maddeler **doğrulanacak**.
- Reddit r/Homebrewing doğrudan bulunamadı (arama Reddit sonucu döndürmedi). Kullanıcı şikayetleri App Store yorumlarından ve HomebrewTalk arama özetlerinden geliyor; sistematik anket değil.
- App Store yorum örnekleri küçüktür (Brewfather UK sayfasında 4 yorum görünüyor, hepsi olumlu); bu bir şikayet yokluğunu kanıtlamaz.

---

## 1. Özet

Rakiplerin hiçbiri Ev modundaki sade tarif ve demleme günü akışı ile İşletme modundaki çok kullanıcılı üretim, kalite ve lot izlenebilirliğini aynı üründe sunmuyor. Brewfather hem ev hem ticari kullanım için tasarlandığını söylüyor, ancak çok kullanıcılı erişim bugün **aynı hesabın ortak girişiyle** yapılıyor; rol, onay ve denetim kaydı bulunmuyor. Bizim en güçlü fark alanlarımız: roller ve denetim kaydı, lot ve CIP kaydı, kâğıt ile ekran arasında QR/tarama köprüsü, 3 parti kuralıyla kalibrasyon asistanı, beklenen eğriye göre fermantasyon uyarısı ve "Nasıl hesaplandı" şeffaflığı. Fiyat ve veri çıkışı da fark yaratabilir: Brewfather'da tam dışa aktarma ve içe aktarma yalnız Premium'da; planımız hiçbir katmanda dışa aktarmayı kapatmıyor. Ev biracısı tarafında Brewfather (4.9 puan, 30 dolar/yıl civarı fiyat, 20'den fazla cihaz, 30 günlük deneme) güçlü ve yerleşik; onu tek başına özellik sayısıyla geçmek zor, geçiş maliyetini düşüren içe aktarma (parti, stok, ekipman dahil) ve doğruluk güveni daha etkili. En büyük planda eksik: Brewfather Premium Plus ve FoamByte yapay zekâ ile tarif üretimi sunuyor veya duyuruyor; planda bu karar açık (analitik model 18 ve AI tarif üreticisi). Ayrıca "ücretsiz katman ve fiyat" kararı hâlâ açık ve rakipler net rakamlarla (Brewfather 25 tarif/25 parti) konumlanıyor. Bu rapor çalışan ürün karşılaştırması değildir: bizim tarafımız plan ve spesifikasyondur, kodlanmış özellik değildir.

---

## 2. Özellik matrisi

Gösterim: **P** = planlı, henüz çalışan ürün yok. **?** = doğrulanacak. Tilt, iSpindel gibi cihaz adları kısaltılmadı. "Fetch özeti" = sayfa özeti, elle kontrol edilmeli.

| Alan | Biz (planlı) | Brewfather | BeerSmith (Mobile + 3/4) | Brew Cloud | Brewtarget | Brewer's Friend | Brew Meister | FoamByte |
|---|---|---|---|---|---|---|---|---|
| Tarif tasarımı | Sihirbaz + editör, hedefle ayarla, tarif kontrolü (P) | Canlı OG/FG/ABV/IBU/renk; 4 IBU formülü; sürümleme (Premium) | Çok derin; Adjust, Convert, Scale, Substitute, Check Recipe | OG/FG/ABV/IBU/SRM, ölçekleme, özel malzeme | Editör; hedefe ayarlama yok | All-grain, BIAB, extract, partial mash; seçilebilir formül | Tarif ve parti yönetimi; derinlik ? | Recipe Studio, canlı |
| Hesap doğruluğu | ADR + altın test + "Nasıl hesaplandı" (P) | Verim ve boil gravity şikayeti (arama özeti, ?) | Sektör referansı | Tahmin ve çökme şikayeti (App Store) | Açık kaynak, birim testli | Su ve pH hesabı övülüyor (arama özeti, ?) | ? | ? |
| Su kimyası | İyon, pH, NNLS, lab raporu (P) | Su profili, tuz, mash pH | Araç var; arayüz eleştirisi (arama özeti, ?) | Su hesaplayıcı var | Kısmi (Water Adjuster) | Tuz, asit, pH; "kullanımı zor" yorumu | ? | Hesaplayıcı var |
| Mash | Adımlar, dekoksiyon, strike, BIAB (P) | Mash profili, mash pH | Mash Wizard ve araçlar | ? | Mash Wizard (no/fly/batch) | Mash sıcaklık kaydı | ? | ? |
| Demleme günü ve sayaç | Adım adım rehber, Live Activity, kâğıt + QR (P) | Rehber, zamanlayıcı; cihazlar arası senkron hatası geçmişte | Mash/kaynatma/whirlpool sayacı | Talimat ve sayaç | Çoklu sayaç | Sayaç var; "başlatamadım" yorumu | Yok / ? | Yok |
| Fermantasyon takibi | Grafik, tahmin, eğriye göre uyarı (P) | Grafik, uyarı, elle kayıt | Okuma grafiği, Tilt (CSV/Sheets) | Gravite ve sıcaklık günlüğü | Yok | Grafik, uyarı | Ana odak: ölçüm ve parti | Parti takibi, IoT yolda |
| Cihaz entegrasyonu | Tilt, iSpindel, RAPT + webhook (P); sensörler İşletme 2 | 20'den fazla: Tilt, RAPT, iSpindel, Grainfather, Brewtools, Plaato, Custom Stream, REST API | Tilt (dolaylı) | ? | Yok | Tilt, iSpindel | EasyDens, SmartRef, pH2Go, Tilt, RAPT Pill, iSpindel | IoT geliştiriliyor (Tilt) |
| Envanter ve stok | Basit stok (Ev); lot, FIFO, COA (İşletme) (P) | Stok + maliyet (Premium) | Var; mobilde stok senkronu yok (yorum) | Yok / ? | Lot bazlı stok | Var | Yok | Geliştiriliyor |
| Alışveriş listesi | Var (P) | Var | Var | ? | 5.3.0'a duyuruldu | ? | Yok | ? |
| İçe/dışa aktarma | BeerXML, BeerJSON, .bsmx, Brewfather, BF; CSV, PDF (P) | BeerXML, JSON tam yedek (Premium) | .bsmx, BeerXML, CSV | BeerXML | BeerXML, BeerJSON, DotBeer | BeerXML ? | PDF, CSV dışa aktarma | CSV ve standart biçimler ? |
| Çok kullanıcı ve roller | Roller, onay, denetim kaydı (İşletme) (P) | Ortak giriş paylaşımı | Yok | Yok | Paylaşılan PostgreSQL (teknik) | ? (Pro katman, ?) | Meister Cloud hesabı | Ücretli planda ekip |
| İşletme özellikleri | Tank, lot, CIP, kalite, mevzuat raporu (P) | Ticari kullanıma uygun denir; özel modül yok | BeerSmith Pro lisansı var; içerik ? | Yok | Yok | Pro katman; içerik ? | Laboratuvar odaklı | Tank View testte |
| Çevrimdışı ve demleme günü | İnternetli; demleme paketi yerelde (P) | Cihaz başına isteğe bağlı çevrimdışı ayar (varsayılan kapalı) | Masaüstü yerel; web ve bulut seçimli | ? | Tamamen yerel | Mobilde çevrimdışı brew day (App Store) | ? | Web |
| Platformlar | iPad, iPhone, web (P) | iOS, Android, web (PWA) | Windows, Mac (4: Mart 2026), web, iOS/Android | iOS (Android ?) | Masaüstü | Web, iOS (Android ?) | iOS, iPad, Watch, Mac, web | Web (mobil yolda) |
| Fiyat ve deneme | 1 ay tam deneme; fiyat açık karar | Ücretsiz 25 tarif/25 parti; Premium ~29,99/yıl; Plus ~49,99/yıl; 30 gün deneme | Mobil 4,99 $; Gold 19,95 $/yıl (yeni fiyat); Basic tek seferlik | Ücretsiz + reklam; reklamsız 2,99 $ | Ücretsiz | Ücretsiz; Premium 5,99/ay veya 41,99/yıl; Plus 134,99/yıl; Pro 539,99/yıl | Ücretsiz; abonelik; fiyat ? | Ücretsiz katman; limitler tutarsız (?) |
| UX | HIG, iPad-first (P) | Modern, hızlı; yüksek puan (4,9) | Eski, karmaşık arayüz eleştirisi | Hafif; çökme yorumu | Masaüstü, eski | Sıkışık mobil, ekran taşması yorumu | Sade, ölçüm odaklı | Yalın |

Kaynaklar sıra numarasıyla Bölüm 7'dedir. Brewtarget sütunu `06-ozellik-veri-modeli-karsilastirma.md` dosyasından.

---

## 3. Rakiplerin güçlü yanları ve kullanıcı şikayetleri

### Brewfather
- **Güçlü:** 20'den fazla cihaz entegrasyonu (Brewtools, Tilt, RAPT, iSpindel, Grainfather, SmartPID, Plaato, Custom Stream, REST API ve webhook). Kaynak: [brewfather.app](https://brewfather.app/).
- **Güçlü:** Net fiyat ve katman yapısı: ücretsiz 25 tarif/25 parti; Premium sınırsız + stok maliyeti, cihazlar, içe/dışa aktarma, PDF, API, sürümleme, özel stil kılavuzu, silinenleri geri getirme; Premium Plus yapay zekâ asistanı (ayda 500 kredi), 5 GB dosya. 30 günlük deneme, kart gerekmez. Kaynak: [docs.brewfather.app/account/subscription](https://docs.brewfather.app/account/subscription).
- **Güçlü:** iOS uygulaması 4,9/5 (267 puan); yorumlar tarif araçlarını, su hesaplayıcısını, cihaz entegrasyonunu ve geliştirici desteğini övüyor (yalnız 4 yorum görünüyor). Kaynak: [App Store (UK)](https://apps.apple.com/gb/app/brewfather/id1488585822).
- **Zayıf:** Çok kullanıcılı kullanım "aynı hesabı ortak giriş ile kullanma"; rol yok. "Ticari bira için tasarlandı" deniyor, ayrıntı yok. Kaynak: [FAQ](https://docs.brewfather.app/faq).
- **Zayıf:** Tam dışa aktarma (JSON) ve içe aktarma yalnız Premium ve web uygulamasında; çevrimdışı destek cihaz başına ayardır ve varsayılan kapalıdır. Kaynak: [Data Safety](https://docs.brewfather.app/data-safety).
- **Zayıf (geçmiş hatalar):** Sürüm notları parti değişikliğinin kaydedilmemesi, cihazlar arası sayaç senkron kaybı, atlanan veya tekrarlanan adım uyarısı hatalarını gösteriyor. Kaynak: [App Store (UK)](https://apps.apple.com/gb/app/brewfather/id1488585822).
- **Şikayet (arama özeti, doğrulanacak):** Verim girdisi verildiğinde OG ve pre-boil tahmininin 4–5 puan yüksek çıkması; "conversion efficiency" girişinin olmaması; renk tahmininin tutmaması ([HomebrewTalk arama sonuçları](https://www.homebrewtalk.com/threads/brewfather-efficiency-sg-predictions-seem-off.674346/post-8766210), sayfa açılamadı). Brewfather kendi belgesinde ekipman profilinin en etkili ayar olduğunu ve her partiden sonra elle güncellenmesi gerektiğini yazıyor; kalibrasyon önerisi otomatik değil. Kaynak: [workflow best practices](https://docs.brewfather.app/brewing-knowledge/brewfather-workflow-best-practices).

### BeerSmith Mobile ve BeerSmith 3/4
- **Güçlü:** Tarif ve araç derinliği (Adjust, Convert, Check Recipe, 29 araç, mead/cider/seltzer, takvim, notlar, arşiv). Kaynak: `06-ozellik-veri-modeli-karsilastirma.md`.
- **Güçlü:** BeerSmith 4 Windows ve Mac için 19 Mart 2026'da çıktı; yeni veritabanı, yerel veya web depolama seçimi, pH takibi, yeniden yazılan oturum özelliği. Kaynak: [BeerSmith blog, 19 Mart 2026](https://beersmith.com/blog/2026/03/19/beersmith-4-available-now-for-windows-and-mac/) (arama özeti) ve [Improved Brewing Features](https://beersmith.com/blog/2026/03/14/improved-brewing-features-in-beersmith-4/).
- **Fiyat:** Yeni Gold 19,95 $/yıl; mevcut aboneler eski fiyatta kalıyor; Gold, Platinum ve Pro için BeerSmith 4 ücretsiz, Basic yükseltmeli. Kaynak: [BeerSmith 4 and Pricing Updates](https://beersmith.com/blog/2026/01/03/beersmith-4-and-pricing-updates/).
- **Zayıf:** BeerSmith Mobile 4,99 $, 3,1/5 (32 puan), sürüm geçmişinde son kayıt 3.1.09 (11 Ekim 2020); stok masaüstüyle senkronlanmıyor, su kimyası aracı mobilde yok (yorumlar). Kaynak: [App Store](https://apps.apple.com/us/app/id640670118).
- **Zayıf:** Mobil sayfada deneme belirtilmiyor; web sürümü BeerSmithRecipes.com'da denenebilir. Kaynak: [beersmith.com/mobile](https://beersmith.com/mobile/).
- **Şikayet (arama özeti, doğrulanacak):** "Karmaşık arayüz", öğrenme eğrisi, su aracı eleştirisi; artı: çevrimdışı kullanım ve tek seferlik ödeme ([HomebrewTalk](https://www.homebrewtalk.com/threads/brewers-friend-vs-beersmith.655758/latest), sayfa açılamadı).

### Brew Cloud
- **Güçlü:** Ücretsiz (reklamlı); reklamı kaldırma 2,99 $; yaklaşık 12 hesaplayıcı, parti takibi, fermantasyon grafiği, BeerXML, BJCP stilleri, topluluk tarifleri; iOS puanı 4,6/5 (353 puan). Kaynak: [App Store](https://apps.apple.com/us/app/brew-cloud/id1254193793).
- **Zayıf:** Yorumlarda tarif düzenlerken çökme (OG ayarında "all grain"e geçip kapanma, Ocak 2025); parti özellikleri yalnız gravite ve sıcaklık günlüğü (2021 yorumu). Aynı kaynak.
- **Eksik/doğrulanacak:** Android sürümü ve BIAB desteği sayfada görünmedi; "iOS, Android" iddiası doğrulanamadı.

### Brewtarget
- **Güçlü:** Ücretsiz, açık kaynak; BeerJSON ve DotBeer desteği, tarif sürüm geçmişi, lot bazlı stok, Boil/Fermentation profilleri. Kaynak: `06-ozellik-veri-modeli-karsilastirma.md` (§1.10, §1.13, §1.7).
- **Zayıf:** Fermantasyon okuması ve grafiği yok, cihaz entegrasyonu yok, bulut ve mobil yok; araç girdileri kaydedilmiyor; hedefe ayarlama araçları yok. Aynı kaynak (§1.2, §1.6, §1.9, §1.12).

### Brewer's Friend
- **Güçlü:** All-grain, BIAB, extract, partial mash; su kimyası (tuz, asit, mash pH); fermantasyon grafiği, Tilt ve iSpindel, uyarı; sayaçlı demleme günü, çevrimdışı kullanım; stok; yazdırma. Kaynak: [App Store](https://apps.apple.com/us/app/brewers-friend/id1580297037).
- **Fiyat:** Premium 5,99/ay veya 41,99/yıl; Plus 14,99/ay veya 134,99/yıl; Pro 49,99/ay veya 539,99/yıl (App Store fiyatı; web fiyat sayfası açılamadı, arama özetinde farklı rakamlar çıktı, doğrulanacak).
- **Şikayet (App Store, 2024–2025):** Sayacı nasıl başlatacağını bulamayıp BeerSmith/Grainfather'a geçen kullanıcı; su aracı "kullanıcı dostu değil"; mobil arayüz sıkışık; malzemelerin kazara değişip geri alınamaması; portrede üst üste binen alanlar. Aynı kaynak.
- **Şikayet (arama özeti, doğrulanacak):** Yıllık ücretin yüksekliği ve arayüz düzeni ([HomebrewTalk](https://www.homebrewtalk.com/threads/brewfather-or-brewers-friend.694796/post-9153854), sayfa açılamadı). Reklam şikayeti yalnız üçüncü taraf özetinde var (güvenilirliği düşük).

### Brew Meister (Anton Paar ConsumerTec)
- **Güçlü:** EasyDens, SmartRef ve pH2Go ölçüm cihazlarıyla bağlantı; Tilt, RAPT Pill, iSpindel; parti karşılaştırma, tam metin arama, PDF/CSV dışa aktarma; iPhone, iPad, Apple Watch, Mac, web (Meister Cloud); 4,7/5 (441 puan). Kaynak: [mwm.ai Brew Meister özeti](https://mwm.ai/apps/brew-meister-measure-manage/1521627812) (App Store sayfasının üçüncü taraf aynası).
- **Zayıf/doğrulanacak:** Ücretsiz indirme, aylık ve yıllık abonelik; fiyat belirtilmiyor. Tarif tasarım derinliği (formüller, su, mash) sayfada görünmüyor; ürün ölçüm ve parti kaydına odaklı görünüyor. Yazılı yorum görünmüyor.
- **Not:** Tür kapsamı bira, kombucha ve hard seltzer. Kaynak: aynı sayfa.

### FoamByte (mevcut raporlardan)
- **Güçlü:** Recipe Studio, Brewing Profiles ve Batch Tracking canlı; ücretsiz planda batch takibi; web erişimi (Windows, Mac, Linux, Chromebook); ekip daveti ve izinler ücretli planda. Kaynak: `foambyte-karsilastirma.md`, `foambyte-faq-karsilastirma.md`.
- **Zayıf/duyuru:** Tank View testte, envanter ve IoT yolda, mobil uygulama geliştirme aşamasında; AI tarif oluşturucu "yakında". Kaynak: `foambyte-site-geneli-karsilastirma.md`.
- **Belirsiz:** Ücretsiz plan limitleri kaynaklar arasında tutarsız (100 tarif/5 batch ve 20 tarif/2 batch); rakam kullanılmadı, doğrulanacak.

---

## 4. Geçebileceğimiz alanlar (sıralı)

Sıralama: rakiplerdeki boşluğun net kanıtı, bizim avantajımızın planda yazılı olması ve hedef kitlede etkisi.

1. **Veri taşınabilirliği ve geçiş kolaylığı (içe aktarma dahil parti, stok, ekipman).**
   - *Rakip boşluğu:* Brewfather'da dışa aktarma ve içe aktarma yalnız Premium, tam JSON yedeği yalnız web'de; yeni kullanıcı için "başka uygulamadan taşıma" ücretli katmanın içinde kalıyor.
   - *Bizim avantaj:* Plan "hiçbir katmanda dışa aktarma kapatılmaz" diyor; içe aktarma eşleştirme adımı (kaynak IBU ile bizim IBU yan yana) planlı.
   - *Kanıt:* [Data Safety](https://docs.brewfather.app/data-safety), [abonelik sayfası](https://docs.brewfather.app/account/subscription). Brewfather'ın dışa aktarma kapsamı recipes, batches, profiles, inventory; biz bunların tamamını içeri alırsak geçiş maliyeti düşer.
2. **İşletme için gerçek çok kullanıcılı yapı (rol, onay, denetim kaydı, lot, CIP).**
   - *Rakip boşluğu:* Brewfather ortak giriş ile; BeerSmith ve Brewtarget tek kullanıcı; FoamByte ekibi ücretli planda yalnız davet ve izin düzeyinde. Küçük işletmeye yönelik kalite ve izlenebilirlik modülü incelenen kaynaklarda görünmedi (doğrulanacak).
   - *Bizim avantaj:* Roller, denetim kaydı, serbest bırakma, CIP, geri çağırma raporu planlı (İşletme çekirdek).
   - *Kanıt:* [Brewfather FAQ](https://docs.brewfather.app/faq), `foambyte-faq-karsilastirma.md`.
3. **Kalibrasyon asistanı ve hesap şeffaflığı (doğruluk güveni).**
   - *Rakip boşluğu:* En sık şikayet "tahmin tutmuyor". Brewfather belgesi profili elle güncellemeyi öğütlüyor; otomatik öneri görülmedi. Forumda verim girdisinin 4–5 puan yüksek tahmin verdiği anlatılıyor (arama özeti, doğrulanacak).
   - *Bizim avantaj:* 3 parti kuralı, kullanıcı onayı olmadan profil değişmez, "Nasıl hesaplandı" sayfası, altın test seti; dönüşüm ve süzme verimi ayrımı (şu an yalnız İşletme).
   - *Kanıt:* [workflow best practices](https://docs.brewfather.app/brewing-knowledge/brewfather-workflow-best-practices), [HomebrewTalk](https://www.homebrewtalk.com/threads/brewfather-efficiency-sg-predictions-seem-off.674346/post-8766210).
4. **Güvenilir demleme günü: sayaç, bildirim ve kâğıt köprüsü.**
   - *Rakip boşluğu:* Brewfather geçmişte cihazlar arası sayaç ve adım uyarısı hataları yaşadı; Brewer's Friend'de bir kullanıcı sayacı başlatamayıp ürünü bıraktı. Hiçbir rakipte QR ile kâğıt formu partiye bağlayan akış görülmedi (doğrulanacak).
   - *Bizim avantaj:* Live Activity, Dynamic Island, sayaç isteğe bağlı, elle saat girişi, brew sheet + QR + VisionKit tarama.
   - *Kanıt:* [Brewfather App Store sürüm notları](https://apps.apple.com/gb/app/brewfather/id1488585822), [Brewer's Friend App Store](https://apps.apple.com/us/app/brewers-friend/id1580297037).
5. **Beklenen eğriye göre fermantasyon uyarısı ve az okumayla tahmin.**
   - *Rakip boşluğu:* Brewfather ve Brewer's Friend yapılandırılabilir eşik uyarısı sunuyor; tahmin eğrisinden sapma görülmedi (doğrulanacak). Brewfather belgesi elle okuma önermeye devam ediyor.
   - *Bizim avantaj:* Fermantasyon bitiş tahmini ve güven aralığı, eğriden sapma uyarısı, cihazsız partide 2 okumayla tahmin, ölçüm kaynak etiketi.
   - *Kanıt:* [brewfather.app](https://brewfather.app/) ("configurable alerts"), plan › Analitik modeller 1–2.
6. **Tek hesap motoru: tüm istemcilerde aynı sayı.**
   - *Rakip boşluğu:* Çok platformlu rakiplerde istemci tarafı hesap; senkron hataları geçmişte raporlandı. BeerSmith Mobile masaüstüyle stok senkronu yok.
   - *Bizim avantaj:* Hesap yalnız sunucuda, tek API; iOS ve web aynı sonucu gösterir.
   - *Kanıt:* [BeerSmith Mobile App Store yorumu](https://apps.apple.com/us/app/id640670118); bu madde plan avantajıdır, ölçülmüş değildir.
7. **Sade Ev modu ve iPad öncelikli arayüz.**
   - *Rakip boşluğu:* BeerSmith arayüz karmaşıklığı, Brewer's Friend sıkışık mobil ve su aracı zorluğu, Brew Cloud çökmeleri.
   - *Bizim avantaj:* HIG, mod ayrımı, sihirbaz, "Tarif kontrolü" ile tek dokunuş düzeltme.
   - *Kanıt:* [Brewer's Friend App Store](https://apps.apple.com/us/app/brewers-friend/id1580297037), [Brew Cloud App Store](https://apps.apple.com/us/app/brew-cloud/id1254193793). Bu üstünlük yalnız prototip ve beta ile kanıtlanır.
8. **Reklamsız, açık fiyat ve tam deneme.**
   - *Rakip boşluğu:* Brew Cloud reklamlı; BeerSmith denemede hesap kısıtı şikayeti planda not edilmiş; Brewer's Friend en yüksek katman 539,99 $/yıl.
   - *Bizim avantaj:* 1 ay tam deneme, deneme sonrası salt okunur ve dışa aktarılabilir.
   - *Kanıt:* [Brew Cloud](https://apps.apple.com/us/app/brew-cloud/id1254193793), [Brewer's Friend](https://apps.apple.com/us/app/brewers-friend/id1580297037). Brewfather da 30 gün deneme veriyor; burada yalnız eşitlik, fark değil.

---

## 5. Planda eksik olanlar (rakiplerde var, bizde yok)

Önem: ABD/İngiltere ev biracısı ve küçük bira fabrikası için.

| Özellik | Hangi rakipte | Önem | Öneri |
|---|---|---|---|
| Yapay zekâ ile tarif üretme ve düzenleme; "stoğumdan ne demlerim" | Brewfather (Premium Plus); FoamByte (yakında) | Orta (hızla yükseliyor) | **MVP'ye ekleme; Ev 2. sürümde karar.** Kural tabanlı "stoktan ne demlenir" (LLM'siz) Ev 2'ye eklenir; LLM'li üretim ürün kararı. Gerekçe: doğruluk riski, hesap motoru sonucu LLM'e bırakılamaz. |
| Herkese açık tarif kitaplığı | Brewfather, Brewer's Friend, Brew Cloud, BeerSmith | Orta | **Ekleme (kapsam dışı).** Gerekçe: karar topluluk yok; içe aktarma ile telafi. |
| Android uygulaması | Brewfather, Brewer's Friend, BeerSmith | Orta (ABD/İngiltere'de pay büyük) | **Ekleme (kapsam dışı).** Gerekçe: karar; PWA yerine tam web zaten var. |
| Cihaz listesi: Grainfather, Brewtools, SmartPID, BrewPiLess, Brewbrain, MyBrewbot | Brewfather | Orta | **Sonra ekle.** Gerekçe: Ev MVP için Tilt, iSpindel, RAPT ve genel webhook yeter; kalan cihazlar webhook ile de bağlanabilir. |
| Brewfather uyumlu "Custom Stream" biçimi (aynı adrese veri gönderen cihazlar) | Brewfather | Yüksek (geçiş maliyeti) | **MVP'ye ekle.** Gerekçe: cihaz ayarını yalnız adres değiştirerek taşımak ucuz ve güçlü; biçim doğrulanacak. |
| Genel REST API ve webhook (tüketici) | Brewfather (Premium) | Düşük-Orta | **Sonra.** Gerekçe: planda API İşletme 2. sürüm; hobi cihazı webhook'u (alım) zaten Ev MVP'de. |
| Silinenleri geri getirme ve değişiklik geçmişi | Brewfather (Premium), Brewtarget (yumuşak silme) | Orta | **MVP'ye ekle.** Gerekçe: kazara silme/değişiklik şikayeti (Brewer's Friend yorumu); okunan plan bölümlerinde yok, planın başka yerinde varsa doğrulanacak. |
| Parti veya tarif için dosya ve görsel depolama (5 GB) | Brewfather (Plus) | Düşük | **Ekleme (planda parti fotoğrafı var).** Gerekçe: fotoğraf Ev MVP'de; ek depolama katman kararına bağlı. |
| Elle gravite-sıcaklık için dijital ölçüm cihazı içe aktarma (EasyDens, SmartRef, pH2Go) | Brew Meister | Düşük-Orta | **Sonra.** Gerekçe: küçük işletme laboratuvarına faydalı; CSV içe aktarma ucuz. |
| Apple Watch uygulaması | Brew Meister | Düşük | **Sonra (planda opsiyonel).** Gerekçe: karar zaten ileride. |
| Kombucha ve diğer tür kapsamı | Brew Meister, Brewtarget (tür olarak) | Düşük | **Ekleme.** Gerekçe: bira, mead, cider, seltzer yeterli. |
| Mead besin ve sülfit araçları | BeerSmith | Düşük-Orta | **Sonra.** Gerekçe: planda mead/şarap formülleri var; besin ve sülfit hesabı yok. |
| Brew sheet şablon düzenleme, 213 değişkenli rapor şablonu | BeerSmith | Düşük | **Sonra (planda Ev 2'de brew sheet özelleştirme var).** |
| Not ve görev listesi | BeerSmith | Düşük | **Ekleme.** Gerekçe: Bugün ve bildirim kutusu görevleri karşılıyor. |
| Yerel depolama seçeneği (bulut yerine) | BeerSmith 4, Brewtarget | Orta (ileri kullanıcı) | **Ekleme.** Gerekçe: karar internetli çalışma; demleme paketi ile telafi. |
| Linux ve Chromebook tarayıcı desteği | Brewfather (PWA), FoamByte | Düşük-Orta | **MVP'ye tarayıcı matrisi ekle.** Gerekçe: React web zaten tarayıcıda çalışır; destek ve test sözü yazılmalı. |
| Ücretsiz katman ve katman matrisi | Brewfather, Brewer's Friend, FoamByte, Brew Cloud | Yüksek | **MVP'ye ekle (karar).** Gerekçe: rakipler rakamla konumlanıyor; plan bu kararı açık bırakıyor. |

---

## 6. Plan güncelleme önerileri

"Karar gerekli" işareti taşıyan maddeler ürün sahibinin (Oğuz) kararını bekler; diğerleri CLAUDE.md'ye göre doğrudan uygulanabilir (planda değişiklik kopya üzerinde değil, asıl kaynakta yapılır).

1. **Kullanıcı şikayetlerinden gelen kararlar › Deneme süresi (katman matrisi).** *Karar gerekli.*
   - *Şimdi:* Fiyat ve ücretsiz katman sınırı açık; yalnız "dışa aktarma kapatılmaz" ve "1 ay tam deneme" yazılı.
   - *Öneri:* Katman matrisi taslağı yaz: tarif, parti, cihaz, kullanıcı, depolama ve destek sınırı. Referans noktaları: Brewfather ücretsiz 25 tarif/25 parti, Premium yaklaşık 29,99/yıl; BeerSmith Gold 19,95/yıl; Brewer's Friend Premium 41,99/yıl. Ev için sınırın tarif sayısından çok cihaz ve sürüm gibi alanlarda konması önerilir (veri sahipliği iddiasını korur).
   - *Etki:* Abonelik ekranı (Ev MVP) ve beta mesajı bu karara bağlı; karar beta öncesi gerekir.
2. **Ek özellikler (yeni satır): "Başka uygulamadan tam taşıma".** *Karar gerekli (Ev MVP kapsamı).*
   - *Şimdi:* Tarif sihirbazı yalnız tarif içe aktarır; Brewfather ve Brewer's Friend dışa aktarımları listelenmiş ama parti, stok ve ekipman geçişi belirtilmemiş.
   - *Öneri:* Brewfather JSON yedeğinden tarif, ekipman, stok ve geçmiş partileri içe aktar (Brewfather dışa aktarması bunları içeriyor); BeerSmith `.bsmx` için aynı. Eşleştirme adımında OG ve renk farkını da IBU gibi yan yana göster.
   - *Etki:* Ev MVP'de tarif ve ekipman, Ev 2. sürümde parti ve stok; beta kullanıcılarını getirmek için kritik.
3. **Eksik hesaplar / Ek özellikler: dönüşüm verimi ile süzme verimini Ev'de de ayır.** *Karar gerekli.*
   - *Şimdi:* Dönüşüm ve süzme verimi ayrımı yalnız İşletme çekirdekte (`formuller/08` §4).
   - *Öneri:* Ekipman profilinde "ileri" ayar olarak Ev'de açılabilir yap; Brewfather'da dönüşüm verimi girişi eksik görünüyor (forum, doğrulanacak).
   - *Etki:* Kalibrasyon ekranı (Ev MVP) küçük genişleme; hesap spesifikasyonu zaten var, istemci işi.
4. **Analitik modeller (18) ve yeni "AI tarif" kararı.** *Karar gerekli.*
   - *Şimdi:* Doğal dille soru sorma "karar gerekli, ilk sürüm için gerekmez"; AI tarif üretimi plan kapsamında hiç yok.
   - *Öneri:* İkiye böl: (a) kural tabanlı "stoktan ne demlenir" ve "benzer tarif" (Ev 2. sürüm, LLM'siz); (b) LLM ile tarif taslağı ve soru-cevap (Ev 2. sürüm sonrası, ayrı karar). LLM yalnız taslak malzeme listesi önerir; OG, IBU, renk Rust motorunda hesaplanır.
   - *Etki:* Ev MVP değişmez; Ev 2. sürüm kapsamına bir madde, risk notu eklenir (maliyet, gizlilik, doğruluk).
5. **Cihaz entegrasyonu ilkeleri (Plaato ifadesi).**
   - *Şimdi:* "Plaato üretimden kalktı, Brewfather desteği kaldırdı" yazıyor.
   - *Öneri:* Brewfather ana sayfasında Plaato hâlâ listeleniyor; ifadeyi doğrulanana kadar "Plaato üretimden kalktı" olarak daralt veya kaynağıyla belirt. Cihaz webhook'una "Brewfather Custom Stream uyumlu biçim" ekle (biçim doğrulanacak).
   - *Etki:* Ev MVP webhook işinde küçük ek.
6. **Demleme günü (bitti ölçütü).**
   - *Şimdi:* Sayaç ve Live Activity tarif edilmiş; cihazlar arası senkron ve bildirim güvenilirliği için test yazılı değil.
   - *Öneri:* iOS ve web için "iki cihazda sayaç tutarlılığı, atlanan veya tekrarlanan uyarı yok" testini bitti ölçütüne ekle (Brewfather sürüm notlarında bu hatalar var). Demleme paketinde sayaç ve ölçüm girişinin çevrimdışı çalıştığını doğrulayan test yaz.
   - *Etki:* Ev MVP kabul testleri; süre eklemez, test ekler.
7. **Tarif editörü ve stil kılavuzları / Sürüm kuralı: silinenleri geri getir.**
   - *Şimdi:* Sürüm ve arşiv kuralı var; silinen tarif, parti ve malzemeyi geri getirme ve alan değişikliği geçmişi okunan bölümlerde yazılı değil.
   - *Öneri:* 30 günlük "Son silinenler" ekranı ve parti/tarif için değişiklik geçmişi ekle (Brewfather bunu Premium'da, Brewtarget yumuşak silmeyle yapıyor). Planın başka bölümünde varsa tekrar yazma, doğrula.
   - *Etki:* Ev MVP'ye küçük ekran ve veri modeli kuralı (yumuşak silme bayrağı).
8. **Mod ayrımı ve Platformlar: tarayıcı matrisi.**
   - *Şimdi:* Web "Mac ve PC" olarak yazılı; Linux ve Chromebook belirtilmemiş.
   - *Öneri:* "Modern tarayıcılar (Chrome, Safari, Firefox, Edge; Windows, macOS, Linux, ChromeOS)" yaz ve CI'a tarayıcı testi ekle. Android hâlâ kapsam dışı.
   - *Etki:* Web aşamasına küçük QA işi; FoamByte'ın web kapsamıyla eşitlik.
9. **Ek özellikler: dijital ölçüm cihazı içe aktarma (CSV).**
   - *Şimdi:* Refraktometre aracı elle girişle çalışıyor; Brew Meister EasyDens/SmartRef/pH2Go bağlıyor.
   - *Öneri:* İşletme 2. sürümde "ölçüm cihazı CSV içe aktarma" ekle (laboratuvar kayıtları ekranına).
   - *Etki:* İşletme 2. sürüm, düşük öncelik.
10. **Hedef kullanıcılar ve iki mod: konumlandırma cümlesi.**
    - *Şimdi:* "Ana rakip Brewfather" yazıyor.
    - *Öneri:* Konumlandırmayı üç kanıtlı farkla yaz: (1) gerçek çok kullanıcılı İşletme modu (Brewfather ortak giriş), (2) veri çıkışı her katmanda açık, (3) kalibrasyon ve "Nasıl hesaplandı" ile doğruluk güveni. Her cümleye bu raporun kaynağı bağlanır; ürün çalışana kadar "planlı" denir.
    - *Etki:* Beta duyurusu ve tasarım sistemi metni; kod etkisi yok.
11. **Yol haritası: Ev 2. sürüm listesi.**
    - *Şimdi:* Ev 2. sürüm maddeleri "Ek özellikler" tablosuna dağılmış, içlerinde AI yok.
    - *Öneri:* Ev 2. sürüme "stoktan ne demlenir" ve (karar sonrası) "AI tarif taslağı" ekle; mead besin ve sülfit araçlarını "Sonra" listesine yaz.
    - *Etki:* Yol haritası ve ISLER.md Ev 2. sürüm bölümü genişler; MVP süresi değişmez.
12. **Eksik hesaplar: mead besin ve sülfit.**
    - *Şimdi:* Mead/şarap formülleri BeerSmith'ten alınıyor; besin ve sülfit hesabı eksik hesaplar tablosunda yok.
    - *Öneri:* "Sonra" aşamasında bir satır ekle (BeerSmith'te var, `06` §1.9).
    - *Etki:* Yalnız kayıt; kod yok.

---

## 7. Kaynaklar

Açılan sayfalar (WebFetch ile okundu):

1. Brewfather ana sayfa: https://brewfather.app/
2. Brewfather abonelik belgesi: https://docs.brewfather.app/account/subscription
3. Brewfather SSS: https://docs.brewfather.app/faq
4. Brewfather veri güvenliği: https://docs.brewfather.app/data-safety
5. Brewfather iş akışı: https://docs.brewfather.app/brewing-knowledge/brewfather-workflow-best-practices
6. Brewfather App Store (UK): https://apps.apple.com/gb/app/brewfather/id1488585822
7. BeerSmith Mobile: https://beersmith.com/mobile/
8. BeerSmith Mobile App Store: https://apps.apple.com/us/app/id640670118
9. BeerSmith 4 ve fiyat: https://beersmith.com/blog/2026/01/03/beersmith-4-and-pricing-updates/
10. BeerSmith 4 özellikleri: https://beersmith.com/blog/2026/03/14/improved-brewing-features-in-beersmith-4/
11. Brew Cloud App Store: https://apps.apple.com/us/app/brew-cloud/id1254193793
12. Brewer's Friend App Store: https://apps.apple.com/us/app/brewers-friend/id1580297037
13. Brew Meister (mwm.ai aynası): https://mwm.ai/apps/brew-meister-measure-manage/1521627812
14. Yerel: `karsilastirma/foambyte-karsilastirma.md`, `foambyte-site-geneli-karsilastirma.md`, `foambyte-calculators-karsilastirma.md`, `foambyte-faq-karsilastirma.md`, `06-ozellik-veri-modeli-karsilastirma.md`
15. Yerel: `plan/demleme-plani.md` (görevde belirtilen bölümler)

Açılamayan sayfalar (yalnız arama özeti var, doğrulanacak): brewersfriend.com/pricing ve ana sayfa (403), Brewfather fiyat ve özellik sayfaları (404), HomebrewTalk konuları (403), anton-paar.com wiki (403), prismnews BeerSmith 4 haberi (503), BeerSmith 4 yayın duyurusu (arama özeti). Reddit'e erişilemedi.

**Doğrulanacak olarak işaretlenen bilgiler:**
- Brewfather fiyatları (29,99/49,99 yıllık, fetch özeti; App Store UK ayrıca £2,99/£29,99 ve £4,99/£49,99 gösteriyor, dönem belirtilmiyor).
- Brewer's Friend güncel web fiyatları (App Store ile arama özeti çelişiyor).
- Brew Cloud Android ve BIAB desteği.
- Brew Meister fiyatı ve tarif tasarım derinliği.
- BeerSmith Mobile'ın güncel sürümü (App Store geçmişinde son kayıt 2020) ve BeerSmith 4 mobil planı.
- Brewfather'da Plaato desteğinin durumu (planla çelişiyor).
- Brewfather "Custom Stream" biçimi.
- Brewfather verim/OG şikayetlerinin güncelliği (arama özeti).
- Kalite/izlenebilirlik modülünün hiçbir rakipte olmadığı (yalnız incelenen kaynaklar için doğru).
- FoamByte ücretsiz plan limitleri.

---

## Ek: Craft Beer Wizard (20 Kasım 2026)

Kaynak: [craft-beer-wizard.com/en/free-beer-recipe-generator](https://craft-beer-wizard.com/en/free-beer-recipe-generator) (sayfa açılarak okundu; diğer sayfalar açılmadı).

- **Ne yapıyor:** tarayıcıda, kurulum yok. Kullanıcı birayı serbest metinle tarif ediyor (ya da 73 BJCP stilinden birini, "Freestyle" ya da "Clone a beer" seçiyor); yapay zekâ tahıl, şerbetçiotu programı, mash, maya, su ve fermantasyon önerisi veriyor. OG, FG, ABV, IBU ve SRM "tahıl ve şerbetçiotu listesine göre hesaplanıyor". Model adı, IBU ve renk yöntemi, verim varsayımı yazılmamış.
- **Fiyat:** misafir ayda 3 yapay zekâ tarifi, kayıtsız; ücretsiz hesapta 3 kayıtlı tarif. Pro (düzenleme, ölçekleme, BeerXML, su kimyası, PDF demleme kaydı, topluluk) alfa süresince ücretsiz; sonrası 4,99 CHF/ay veya 20 CHF/yıl.
- **Diğer:** demleme günü sayaçları, sıcaklık kaydı, fermantasyon takibi, envanter, topluluk tarifleri, hesaplayıcılar. Cihaz entegrasyonu yok, yerel uygulama yok, yalnız İngilizce görünüyor.
- **Dürüstlük notları:** "tarifi demlemeden önce gözden geçirin" uyarısı; Brewfather ve BeerSmith karşılaştırması "üretici bilgisi, bağımsız deneme değil" diye işaretli.

**Bizim için anlamı:**
1. **Yapay zekâ ile tarif taslağı artık pazarda var** ve giriş kapısı olarak kullanılıyor (kayıtsız ilk tarif). Onaylı yaklaşımımız aynı ayrımı yapıyor: model yalnız malzeme önerir, sayıları hesap motoru hesaplar. Bizim farkımız hesabın şeffaf ve doğrulanmış olması ("Nasıl hesaplandı", parite testleri, kalibrasyon); onların yöntemi yazılı değil.
2. **Fiyat çıpası düşük:** 20 CHF/yıl (yaklaşık 22–25 $, kur doğrulanacak), Brewfather Premium'a yakın. Katman matrisi taslağında (O8) dikkate alınmalı.
3. **Pazarlama:** "ücretsiz tarif üreticisi" ve tek tek hesaplayıcı sayfaları arama trafiği için kurulmuş. Bu, pazarlama sitesi iş parçacığının konusu (hesaplayıcı sayfaları `web-stratejisi/20-*` içinde var).
4. **"Clone a beer" (bilinen ticari birayı taklit eden tarif):** planda yok. Dil modeli kararıyla birlikte değerlendirilebilir.
5. **Eksikleri bizim güçlü yanımız:** cihaz entegrasyonu, iPhone/iPad uygulaması, İşletme modu ve çok kullanıcı yok.
