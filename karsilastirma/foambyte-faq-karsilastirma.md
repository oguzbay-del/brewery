# FoamByte FAQ ↔ Demleme: yetkinlik ve plan eşleştirmesi

> **Claude için:** Bu rapor, FoamByte FAQ sayfasında listelenen ürün politikaları ve yetkinlikleri ile Demleme repo'sundaki ürün planını karşılaştırır. Repo kapsamındaki her maddeyi mutlaka durum etiketiyle okuyun: **planlı**, **kararı açık**, **kısmen karşılık var** veya **plan kapsamı bulunamadı**. Bunların hiçbiri çalışan özellik olduğu anlamına gelmez.
>
> **Önemli kaynak notu:** FoamByte FAQ içeriği bu incelemede web arama sonuçları üzerinden özetlendi; tam sayfanın her satırı bağımsız olarak doğrulanamadı. Bu nedenle aşağıdaki FoamByte maddeleri ürünün resmî FAQ sayfasına atfedilen özetlerdir; fiyat, plan sınırları ve özelliklerin güncel durumu yayımlamadan önce sayfadan yeniden kontrol edilmelidir.

**İnceleme tarihi:** 7 Kasım 2026  
**FoamByte kaynağı:** [Brewing Software FAQs](https://foambyte.com/faqs/)  
**Repo:** [`oguzbay-del/brewery`](https://github.com/oguzbay-del/brewery)

## Kısa sonuç

Demleme'nin ürün planı, FoamByte FAQ özetinde geçen web erişimi, hesapla kullanım, ücretli ekip işbirliği, batch takibi ve envanter başlıklarının çoğuna **ürün kapsamı olarak** karşılık içeriyor. Demleme planı ayrıca iOS uygulamaları, işletme rolleri, tank/lot izlenebilirliği ve üretim/kalite akışları hedefliyor.

Bununla birlikte, FoamByte'un ücretli/ücretsiz planlara göre belirlediği özellik sınırlarıyla Demleme'nin planı aynı değil. Demleme'nin fiyatlandırma ve ücretsiz katman sınırları açık bırakılmış; işbirliği/parti özelliklerinin hangi abonelik planına ait olacağı repo planında kesinleşmiş görünmüyor. Dahası, README ve `CLAUDE.md` uygulama kodunun henüz yazılmadığını açıkça belirtiyor. Dolayısıyla sonuç **“plan karşılığı var”** olmalı, **“Demleme bu yetenekleri şu an sunuyor”** olmamalıdır.

## Yetkinlik eşleştirmesi

| FoamByte FAQ'da özetlenen konu | FoamByte tarafındaki bilgi (FAQ özeti) | Demleme reposundaki karşılık | Durum / boşluk |
|---|---|---|---|
| **Platform erişimi** | Tarayıcı üzerinden web; Windows, Mac, Linux tarayıcıları ve Chromebook. Mobil uygulama geliştirme aşamasında. | Ürün planı tam özellikli web (Mac/PC) ile iPad/iPhone için SwiftUI uygulamaları hedefliyor; Apple Watch sonraki opsiyonel adım. | **Planlı, kısmen farklı:** web ve iOS planı var. Linux/Chromebook için tarayıcı uyumluluğu açıkça belirtilmemiş. Hiçbir istemci uygulanmış değil. |
| **Ücretsiz ve ücretli katmanlar** | Ücretsiz plan ve ücretli kademeler; ücretsiz planda temel özellikler ve sınırlamalar, ücretli planlarda daha gelişmiş imkânlar. FAQ özetinde batch, işbirliği ve depolama gibi farklar ücretli planlarla ilişkilendiriliyor. | Demleme planında abonelik/ödeme ekranı, bir aylık tam deneme ve deneme sonrası verilerin silinmemesi düşünülmüş. Fiyatlar ve ücretsiz katmanın sınırları açık karar olarak bırakılmış. | **Kısmen planlı / karar açık:** abonelik ve deneme planı var; fiyat ve ücretsiz/ücretli özellik matrisi kararlaştırılmamış. FoamByte'ın tier sınırlarıyla birebir eşleşme yok. |
| **Hesap zorunluluğu** | Ücretsiz katman dâhil hesap oluşturmak gerekiyor. | Plan ayrı kayıt/şifre yerine Apple, Google veya e-posta bağlantısıyla giriş öngörüyor; ilk açılışta giriş gerekiyor, hesapsız kullanım hedeflenmiyor. | **Planlı, yaklaşım farklı:** hesap şartı örtüşüyor; Demleme'nin giriş yöntemleri ve hesap bağlama akışı ayrıca tanımlanmış. |
| **Kullanıcı verisinin sahipliği** | FAQ özetine göre kullanıcı verisinin sahipliğini koruyor. | Plan veri dışa aktarma ve hesap silme iş akışlarını, gizlilik/güvenlik kontrollerini ele alıyor. Ancak incelenen repo planında FoamByte'ın ifadesiyle aynı açık “veri mülkiyeti kullanıcıda kalır” taahhüdü tespit edilmedi. | **Kısmi karşılık / açık metin eksik:** dışa aktarma ve silme düşünülmüş; hukuki/veri sahipliği taahhüdü açık ürün veya gizlilik metni olarak doğrulanmalı. |
| **Veri dışa aktarma** | FAQ özeti CSV ve diğer standart biçimlerde dışa aktarmadan bahsediyor. | Plan tarif içe/dışa aktarma (BeerXML, BeerJSON ve kaynak formatlar), veri dışa aktarma, PDF/baskı ve hesap silme taleplerini kapsıyor. Her veri türü için CSV ve tam hesap dışa aktarma garantisi net değil. | **Kısmen planlı:** tarif formatları ve genel dışa aktarma var; CSV kapsamı, tüm verinin taşınabilirliği ve self-service davranışı ayrıca kesinleştirilmeli. |
| **Ekip işbirliği ve izinler** | FAQ özetine göre ücretli planlarda ekip daveti, izinler ve gerçek zamanlı ortak çalışma var. | İşletme modu çok kullanıcılı; roller/yetkiler, davetler, ortak cihaz, denetim kaydı ve eşzamanlı düzenleme stratejileri planlanmış. SSE ile canlı güncelleme hedefi de tarif edilmiş. Ev modunda sınırlı paylaşım/davet düşünülüyor. | **Planlı, ücret ilişkisi açık değil:** ekip ve izin altyapısı kapsamda; gerçek zamanlı düzenleme davranışı tasarlanmış. Hangi abonelik planının açacağı, ücret ve eşzamanlılık kapsamı kararlaştırılmalı. |
| **Batch / parti takibi** | FAQ özeti parti numarası atama ve ilerlemeyi izlemeyi ücretli özelliklerle ilişkilendiriyor. | Parti durumları, planlanan/gerçekleşen kayıtları, fermentasyon ölçümleri, tank hareketleri, kalite ve lot izlenebilirliği ürün planında var. | **Planlı, kapsamı geniş:** temel batch takibine açık karşılık var. Ancak şu an çalışan özellik değil; ücretsiz/ücretli sınır belirlenmemiş. |
| **Envanter** | FAQ özeti stok miktarı, konum ve tüketimin zaman içinde takibini anlatıyor. | Ev modunda basit stok/alışveriş listesi; işletmede lot, tedarikçi, rezervasyon, FIFO ve izlenebilirlik planlanmış. | **Planlı, kapsamı geniş:** envanter hedefi var; konum bazlı stok ve bunların veri modeli/iş akışı ayrıca kontrol edilmeli. |
| **Destek** | FAQ özeti ücretli planlarda öncelikli destekten söz ediyor. | Demleme planında yardım arama, destek talebi, talep yanıtı ve müşterinin iznine bağlı iç destek erişimi tasarlanmış. | **Kısmi karşılık:** destek kanalı planlı; öncelikli destek seviyesi, yanıt süresi/SLA ve hangi abonelik katmanında olduğu tanımlı değil. |

## Demleme planında mevcut karşılıkların dayanağı

- `README.md`: repo'nun plan ve tasarım çalışması olduğunu, uygulama kodunun henüz yazılmadığını söylüyor.
- `CLAUDE.md`: uygulama kodu yazılmaması; işin plan, tasarım, formül spesifikasyonu ve araştırma olduğu belirtiliyor.
- `plan/demleme-plani.md`:
  - platformlar ve mimari: iPad/iPhone SwiftUI, web React/TypeScript;
  - Ev ve İşletme modları;
  - parti/fermantasyon, stok/alışveriş, roller ve yetkiler;
  - hesap, abonelik, deneme süresi, dışa aktarma ve hesap silme;
  - cihaz, veri saklama, güvenlik ve destek akışları.

## Ürün açısından çıkarımlar

1. **Önce ürün planındaki fiyatlandırma kararını kapatın.** Ücretsiz planda tarif, hesap makineleri, batch, envanter ve davetlerden hangilerinin olacağı açık değil. FoamByte FAQ'sındaki sınırlar da tekrar doğrulanmalı.
2. **Veri taşınabilirliğini özellik olarak kesin tanımlayın.** Tarif dışa aktarma ile tüm kullanıcı/parti/ölçüm verisini dışa aktarma farklı kapsamlar. CSV, BeerXML/BeerJSON ve ek dosyaların kapsamı, silme ve yedekleme akışları ayrı yazılmalı.
3. **İşbirliğini plan seviyelerine bağlayın.** Demleme'de işletme rolleri ve denetim kaydı var; fakat ev modu paylaşımı, davet sayısı, gerçek zamanlı ortak çalışma ve abonelik kapıları netleştirilmeli.
4. **Platform kapsamını doğru ifade edin.** FoamByte FAQ'sı web ve geliştirilmekte olan mobil uygulamadan söz ediyor; Demleme iOS + web hedefliyor. Linux/Chromebook erişimi bir web hedefi olabilir, ancak test edilmeden vaat edilmemeli.
5. **Ürün karşılaştırmasını uygulama durumu ile karıştırmayın.** Bu repo'daki kapsam plan ve spesifikasyondur; rakibin çalışan özellikleriyle işlevsel parite kanıtı değildir.

## Sonuç cümlesi

**Demleme repo'sunda FoamByte FAQ'sında geçen platform, hesap, dışa aktarma, ekip işbirliği, batch, envanter ve destek başlıklarının çoğu plan düzeyinde bulunuyor. Fakat fiyatlandırma/özellik katmanları, veri sahipliği taahhüdü, tüm veriyi CSV ile dışa aktarma ve destek SLA'sı açık değil; ayrıca repo henüz çalışan uygulama içermiyor.**

## Kaynaklar

- [FoamByte FAQ](https://foambyte.com/faqs/)
- [Demleme README](https://github.com/oguzbay-del/brewery/blob/main/README.md)
- [Demleme ürün planı](https://github.com/oguzbay-del/brewery/blob/main/plan/demleme-plani.md)
- [Claude çalışma kuralları](https://github.com/oguzbay-del/brewery/blob/main/CLAUDE.md)
