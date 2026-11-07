# FoamByte özellikleri ile Demleme projesi karşılaştırması

> **Claude için:** Bu raporu ürün planı veya uygulama kapsamı üzerinde çalışırken okuyun. Repo içindeki kapsamı, mevcut kod gibi sunmayın: README ve `CLAUDE.md`'ye göre bu depo şu anda plan, tasarım, formül spesifikasyonu ve test vektörleri içeriyor; gerçek uygulama kodu henüz yazılmadı. Karşılaştırma bir özellik/plan karşılaştırmasıdır, çalışan ürünlerin eşdeğerlik testi değildir. Belirsiz veya dış kaynakla doğrulanmamış bilgileri kesin gerçek gibi aktarmayın.

**İnceleme tarihi:** 7 Kasım 2026  
**Referans:** [FoamByte özellikler sayfası](https://foambyte.com/features/) ve sayfada bağlantı verilen [Brewing Profiles](https://foambyte.com/features/brewing-profiles/) ile [Batch Tracking](https://foambyte.com/features/batch-tracking/) sayfaları.  
**Repo:** [`oguzbay-del/brewery`](https://github.com/oguzbay-del/brewery), varsayılan dal `main`.

## Yönetici özeti

Demleme projesinin yazılı ürün planı, FoamByte'ın öne çıkardığı temel alanlarla — tarif oluşturma, reçete/parti hesapları, demleme profilleri ve parti/fermantasyon takibi — büyük ölçüde örtüşüyor. Demleme planı ayrıca iOS ve web istemcileri, ev ve küçük işletme modları, çevrimdışı demleme günü, kapsamlı test vektörleri ve işletme üretim akışları gibi daha geniş bir hedef tarif ediyor.

Ancak kritik ayrım şudur: **bu depo şu anda çalışan bir bira üretim yazılımı değil.** README ve `CLAUDE.md` uygulama kodunun henüz yazılmadığını açıkça belirtiyor. Depodaki HTML/CSS, ürün konsepti ve tasarım sistemi önizlemeleri; Python dosyaları ise formül test/referans uygulamalarıdır. Bu nedenle FoamByte'ın çalışan veya testte olan özelliklerini Demleme'de “mevcut” olarak işaretlemeyin. Bu rapordaki “karşılıyor” değerlendirmeleri, yalnızca plan ve spesifikasyon kapsamındaki karşılığı anlatır.

## Özellik karşılaştırması

| FoamByte özelliği | FoamByte sayfasında anlatılan | Demleme deposundaki karşılık | Değerlendirme |
|---|---|---|---|
| **Recipe Studio / tarif yönetimi** | Tarif oluşturma ve yönetme | Tarif sihirbazı ve editörü; hedef OG/IBU/renk, ölçekleme, tarif sürümleri ve stil kılavuzları planlanmış. Formül dokümanları yoğunluk, renk, IBU, su/pH, maya ve hacim hesaplarını kapsıyor. | **Plan düzeyinde güçlü karşılık.** Henüz çalışan tarif editörü yok. |
| **Batch & Fermentation Tracking** | Partiyi planlamadan demleme, fermantasyon, kondisyon ve karbonasyona kadar izleme; SG, sıcaklık, pH, not ve tarih kaydı | `plan/demleme-plani.md` birden fazla eşzamanlı parti, aşama geçişleri, planlanan/gerçekleşen değerler, ölçüm kaynakları, fermantasyon grafikleri, uyarılar ve cihaz okumaları tarif ediyor. | **Plan düzeyinde güçlü; kapsamı daha ayrıntılı.** Sensör/webhook ve canlı izleme gerçek sistemde henüz uygulanmış değil. |
| **Brewing Profiles** | Ekipman, brewhouse, hacim, verim ve kayıplar için profil tanımlama | Ekipman/brewhouse profilleri, tanklar, hacim zinciri, verim ve kayıp parametreleri; geçmiş partilerle kalibrasyon önerileri planlanmış. | **Plan düzeyinde güçlü karşılık.** Profil düzenleme ve otomatik kalibrasyon kodu yok. |
| **Brewing Calculators** | Demleme sürecindeki çeşitli hesaplayıcılar | `formuller/` altında OG/FG/ABV, IBU, renk, su/pH, maya, karbonasyon, mash ve hacim spesifikasyonları; `formuller/testler/` içinde Python referans/test vektörleri mevcut. | **Spesifikasyon ve referans testleri mevcut; ürün motoru yok.** Test dosyalarını üretim servisi olarak nitelemeyin. |
| **Tank View** | Tank görünürlüğü, parti takibi ve elle kalite ölçümü; FoamByte sayfasındaki açıklamaya göre test aşamasında | İşletme modu için tank panosu, tank takvimi, tank doluluğu, transfer/bölme/birleştirme ve kalite akışları tasarlanmış. | **Plan düzeyinde daha geniş hedef.** FoamByte'ın testteki özelliğiyle işlevsel karşılaştırma yapılamaz; Demleme'de tank ekranı çalışan ürün değildir. |
| **Küçük üreticiye yönelik yalınlık/erişilebilirlik** | Ev biracısı ve nano/küçük bira fabrikaları için sade ve uygun maliyetli kullanım; ücretsiz sürüm ve ücretli yükseltmeler | Ev ve İşletme modları aynı ürün planında; hedef kitle örtüşüyor. Repo fiyatlandırma, ücretsiz katmanın sınırları ve ürünün gerçek kullanım kolaylığı konusunda doğrulanmış ürün verisi sunmuyor. | **Hedef kitle örtüşüyor; yalınlık ve fiyat avantajı kanıtlanmış değil.** Demleme planının kapsam genişliği karmaşıklık riski yaratıyor. |
| **Envanter / üretim / cihazlar / mobil** | Envanter ve kapsamlı üretim, IoT okumaları ve mobil uygulama FoamByte sayfasında planlanan/yol haritasındaki başlıklar olarak sunuluyor. | Demleme planında stok, lot takibi, üretim panosu, iPhone/iPad SwiftUI uygulaması, web istemcisi ve Tilt/iSpindel/RAPT gibi cihaz entegrasyonları hedeflenmiş. | **Plan düzeyinde kapsam örtüşmesi ve genişleme var.** Hiçbiri bu repo içinde tamamlanmış bir uygulama özelliği değil. |

## Önemli farklar ve ürün konumlandırması

1. **Uygulama durumu, özellik listesinden daha önemli:** FoamByte bir ürün/özellik sitesi olarak işlevleri sunuyor; Demleme deposu ise ürün geliştirme öncesi plan ve tasarım varlığı. Dolayısıyla “bizde bu özellik de var” yerine “planımızda karşılığı var” denmeli.
2. **Demleme'nin hedef kapsamı geniş:** Ev biracılığına ek olarak rol tabanlı işletme kullanımı, lot izlenebilirliği, kalite, onaylar, tank operasyonları ve mevzuat raporları düşünülüyor. Bu, küçük ekipler için daha fazla kapsam sunabilir; fakat ürünün ilk sürümünü geciktirme ve kullanımını karmaşıklaştırma riski de doğurur.
3. **Hesap motorunda test yaklaşımı belirgin:** Depoda formül spesifikasyonları ve Python test vektörleri bulunması güçlü bir hazırlık. Yine de test vektörlerinin FoamByte veya diğer ürünlerle aynı girdilerde doğrulanmış sonuç verdiği varsayılmamalı; kendi dokümanları da bazı sonuçların ürün içi değerlerle doğrulanması gerektiğini belirtiyor.
4. **İşletme özellikleri FoamByte'ın sayfadaki temel anlatımından ileri hedefler:** Demleme'nin planı çok kullanıcılı roller, denetim izi, CIP/kalite, lot ve üretim izlenebilirliği gibi işlevleri hedefliyor. Bunlar plan kapsamı; çalışan veya pazarda doğrulanmış özellikler değiller.
5. **Platform yaklaşımı farklı bir yatırım yükü getiriyor:** Plan native SwiftUI iPhone/iPad ve React web hedefliyor; ortak hesap motoru ve API de öngörülüyor. Bu çoklu istemci hedefi, FoamByte sayfasında öne çıkan yalın ürün vaadinden ayrı olarak önemli geliştirme ve bakım maliyeti doğurabilir.

## Repo kanıtı ve durum sınırları

- `README.md`: depo plan/tasarım çalışması; “uygulama kodu henüz yazılmadı” ifadesi var.
- `CLAUDE.md`: uygulama kodu yazılmaması, işin plan/tasarım/formül araştırması olduğu belirtiliyor.
- `plan/demleme-plani.md`: hedef kullanıcılar, özellik kapsamı, platform ve mimari planı.
- `formuller/`: hesap formülleri için dokümantasyon ve Python referans/test dosyaları.
- `tasarim-sistemi/` ve `arayuz/`: tasarım bileşenleri ve HTML konsepti; bunlar üretim uygulaması olduklarını kanıtlamaz.
- GitHub dil istatistikleri HTML/Python/CSS gösteriyor; Swift, Rust veya TypeScript uygulama kodu görünmüyor. Bu, README'deki henüz kodlanmamış durumuyla tutarlıdır.

## Sonuç

**Stratejik açıdan:** Demleme'nin yazılı planı FoamByte ile aynı ana kullanıcı ihtiyaçlarını hedefliyor ve özellikle küçük işletme üretim akışlarında daha kapsamlı bir ürün vizyonu tarif ediyor. **Ürün olgunluğu açısından:** FoamByte'ın sayfadaki mevcut/testteki işlevleriyle Demleme arasında özellik-paritesi iddiası kurmak için repo kanıtı yok; Demleme şu an plan, tasarım ve hesap spesifikasyonu aşamasında.

En sağlıklı kısa ifade: **“Demleme, FoamByte'ın tarif, profil, hesap ve parti/fermantasyon alanlarıyla örtüşen; ev ve küçük işletmelere yönelik daha geniş bir ürün planına sahip. Ancak bu repo henüz çalışan uygulama değil, dolayısıyla özellikler uygulanmış değil.”**

## Kaynaklar

- [FoamByte — Features](https://foambyte.com/features/)
- [FoamByte — Brewing Profiles](https://foambyte.com/features/brewing-profiles/)
- [FoamByte — Batch Tracking](https://foambyte.com/features/batch-tracking/)
- [Demleme repo README](https://github.com/oguzbay-del/brewery/blob/main/README.md)
- [Demleme ürün planı](https://github.com/oguzbay-del/brewery/blob/main/plan/demleme-plani.md)
- [Claude çalışma kuralları](https://github.com/oguzbay-del/brewery/blob/main/CLAUDE.md)
- [Formül spesifikasyonları dizini](https://github.com/oguzbay-del/brewery/tree/main/formuller)
