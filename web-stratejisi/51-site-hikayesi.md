# GRISTORA web sitesi · Bir partinin hikâyesi

**Editoryal karar:** Ziyaretçi önce üreticinin verdiği kararı görür. Ardından o kararı açıklayan ürün ekranını inceler. Ana sayfadaki mahzen fotoğrafı üretim ortamını kurar. Hemen altındaki gerçek tasarım ekranları yazılımın o ortamda ne işe yaradığını gösterir. Görseller için [ekran kullanım tablosu](52-ekran-kullanim-tablosu.md) esas alınır.

## Ürün planıyla bağ

Profesyonel üretici ana ticari hedef, ev biracısı ikinci ve ayrı ziyaretçi yoludur. Geliştirme sırası farklıdır. [Ürün planı](../plan/demleme-plani.md) önce paylaşılan çekirdeği ve Ev MVP betasını, ardından İşletme modunu tarif eder. Tarif, hesaplama, demleme ve parti kaydı iki hikâyede ortaktır. Ev yolunda bir kişi, kendi ekipmanını ve tekrar demleyeceği tarifi izler. İşletme yolunda aynı kaydı vardiyalar, tanklar, hammadde lotları ve kalite kararları birbirine bağlar.

Her özellik detayının kapsamını [özellik kataloğundaki](../marketing-site/src/data/features.ts) aşamayla eşleştir. Bir ekranın çizilmiş olması sürüm durumunu değiştirmez. Görüşmeler, hesap motorunun altın testleri ve beta sonuçları ürün kapılarıdır. Tarih, fiyat, kesin entegrasyon veya müşteri sonucu eklemek için ürün kararını bekle. [Kamuya açık ürün ifadeleri](41-gristora-public-marketing-claims-guardrails.md)

## Ana sayfanın anlatı akışı

1. **Sabah, tankların önünde.** Mevcut mahzen fotoğrafı ve reçeteden parti kaydına başlığı. Birincil bağlantı bira üreticisinin yolunu açar.
2. **Masa başında hedef.** Mevcut örnek partide hedef 500 litre. Reçete editörünü göster. Altyazı, hedefin hangi kayda ait olduğunu anlatır.
3. **Demleme günü gerçekleşen.** Operatör iPad üzerinde yapılan işi kaydeder. Mevcut örnekte tanka aktarılan 490 litre anlatının parçasıdır. Tasarım ekranında farklı rakam varsa altyazıda ekrandaki rakamı kullan veya iki örneği ayrı adlandır.
4. **Ertesi vardiya.** Fermantasyon özeti ve parti kaydı üzerinden “Bir sonraki kişi nereden devam eder?” sorusunu yanıtla. Mobilde her ekranda tek karar göster.
5. **Kendi yolunu seç.** Profesyonel üreticiyi tank/kalite akışına, ev biracısını reçete ve demleme gününe götür. Sonraki adım mevcut plan karşılaştırmasıdır.

## Sayfa haritası ve işlevi

Her satır İngilizce rota ile onun Türkçe eşine uygulanır. Ekran içi dil ortaktır. Sayfa metni, alternatif metin ve altyazı sayfanın dilindedir.

| Sayfa | Ziyaretçinin sorusu ve anlatı | Görsel kullanımı | Sonraki adım |
|---|---|---|---|
| Ana sayfa | Bir reçeteyi aynı partinin ölçümleriyle nasıl takip ederim? | Mahzen fotoğrafı → editör → iPad demleme günü → parti özeti | Kendi üretim yolunu seç |
| /breweries/ | Vardiya değiştiğinde tankın durumunu kim bilir? | Pano genel görünümü, tek tank detayı, kalite kararı | Kalite veya üretim yönetimi |
| /homebrewers/ | Bir sonraki demlemede neyi tekrarlarım? | Tarif sihirbazı, demleme günü ve iki tarif sürümünün karşılaştırması | Tarif tasarımı |
| /how-it-works/ | Planla, uygula, izle ve kapat adımları nasıl bağlanır? | Takvim, iPad zamanlayıcı, fermantasyon, parti özeti | İlgili özellik detayı |
| /features/ ve detayları | Belirli bir işi nasıl yaparım? | Dizinde görev bağlantıları; detayda tek ekran ve ilgili kontrolün okunabilir yakın planı | İlgili modül |
| /modules/ | Hangi modül hangi kararı kapsar? | Stok için lot, kalite için spesifikasyon, üretim için tank. Kartların hepsine aynı pano görselini koyma | Özellik detayı veya karşılaştırma |
| /technical-principles/ | Bir ölçümün ve birimin anlamını nasıl görürüm? | Hesap aracı ve açıklama penceresi; sonuçtan çok girdiyi ve birimi anlat | Hesaplamalar |
| /compare/, /pricing/, /get-started/ | Benim üretim biçimime hangi kapsam uyuyor? | Karşılaştırmada Ev tarifi ve İşletme panosunu yan yana kullan. Fiyatlandırma ve sihirbazda seçim ile özet ekranına öncelik ver | Plan özeti |
| /faq/ | Ürün aşaması, kapsam ve önizleme hakkında ne bilmeliyim? | Soruyu yanıtlayan özellik detayına bağlantı. Tekrarlanan galeri ekleme | İlgili sayfa |

## Uygulama sırası ve kabul

| Etap | Teslim | İnceleme ölçütü |
|---|---|---|
| 1 · Örnek parti | Ana sayfa ve iş akışı sayfasında aynı hikâyenin TR/EN önizlemesi | Okuyucu hedefi, gerçekleşeni ve sonraki adımı ayırt edebiliyor; kaynak ile altyazı tutarlı |
| 2 · İki üretici yolu | Profesyonel üretim ve ev üretimi sayfaları, ardından öncelikli özellik detayları | Tank, lot ve kalite ile tarif, demleme ve karşılaştırma yolları kendi detaylarına ulaşıyor |
| 3 · Katalog ve karar | Modüller, teknik ilkeler, karşılaştırma, plan ve SSS bağlantıları | Mevcut kapsam etiketleri korunuyor; plan özeti ayrı bir teklif talebi izlenimi vermiyor |
| 4 · Bakım | Yayındaki kullanımların tablosu ve güncelleme provası | Tek bir kaynak değişince bağlı tüm TR/EN konumları bulunabiliyor |

Her etapta 390, 768 ve 1440 px önizleme al. Mobilde ürün arayüzünü küçücük bir tablet çerçevesine sıkıştırmak yerine karar alanını kırp, tam ekrana erişim sun. Genişliği ve yüksekliği baştan ayır, ekran altındaki görselleri gecikmeli yükle. Metin ve bağlantılar görsel olmadan da anlamlı kalsın. Mevcut Astro, bağlantı, erişilebilirlik ve Lighthouse kontrollerini çalıştır. Başarıyı mevcut ölçüm planındaki üretici yolu, özellik detayı ve plan özeti geçişleriyle izle; ilk yayın öncesi değerleri başlangıç noktası olarak kaydet.
