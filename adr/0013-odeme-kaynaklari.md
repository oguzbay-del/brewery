# ADR-0013: Ödeme kaynakları

**Durum:** Önerildi · **Tarih:** 26 Kasım 2026 · **Karar:** Oğuz'un 25.11.2026 tarihli kanal seçimi, uygulama ayrıntıları incelemede · **İş:** B1

## Bağlam

Ev (Home) aboneliği iOS'ta App Store üzerinden, web'de Paddle üzerinden alınabilir. İşletme (Business) aboneliği web'de Paddle üzerinden alınır, koltuk sayısına göre yönetilir. Abonelik durumunun ve kullanım yetkisinin kaydı sunucudadır. [Ürün planı](../plan/demleme-plani.md#abonelik-ve-ödeme) bu kanal ayrımını tanımlar.

- [Apple alım adaptörü](../sunucu/crates/ingest/src/apple.rs) doğrulanmış bildirimi `SubscriptionUpdate` biçimine dönüştürür. İşlemdeki `appAccountToken` çalışma alanını belirler.
- [Paddle adaptörü](../sunucu/crates/billing/src/paddle.rs) bildirimin imzasını doğrular, olay ve ürün bilgilerini ortak biçime eşler. [Alım servisi](../sunucu/crates/ingest/src/lib.rs) adaptörü yapılandırır.
- [Abonelik modeli](../sunucu/crates/billing/src/lib.rs) ortak bildirim biçimini ve `PAID_FEATURE` hakkını tanımlar. [Uygulama katmanı](../sunucu/crates/billing/src/apply.rs) doğrulanmış olaylardan abonelik ve yetki kaydını günceller.

## Karar

### 1. Çalışma alanı başına tek aktif kaynak

Sunucu her çalışma alanı için en fazla bir aktif ödeme kaynağı tutar. Ödeme başlatma ve App Store eşitleme uçları, diğer kaynak aktifse HTTP `409` döndürür. İstemci mevcut kaynağı ve yönetim yerini gösterir.

İkinci kaynağın aktif olduğunu gösteren doğrulanmış webhook gelirse alım servisi olayı kaydeder ve destek kaydı açar. Mevcut kaynak ve yetki korunur. Aynı olay yeniden gelirse ikinci bir destek kaydı açılmaz. İptal ve varsa iade ilgili sağlayıcıdan yürütülür. Sunucu kaynak değişimini işlem içinde denetler ve kayda alır.

### 2. App Store kuralları

- **3.1.1.** Uygulama içindeki dijital özellik veya aboneliğin kilidini açmak için Apple genel olarak uygulama içi satın alma ister. **3.1.1(a)**, ABD mağazasında haricî satın alma bağlantıları için ayrı hükümler içerir. Diğer mağazalarda bağlantı ve çağrı butonu koşulları mağazaya ve Apple yetkilerine bağlıdır.
- **3.1.3(b).** Çok platformlu hizmetlerde web'den alınan içeriğe erişim, ilgili içeriğin uygulama içi satın almayla da sunulması koşuluna tabidir. Ev planı her iki kanalda sunulur. Ev katmanlarının birebir karşılığı ve bölgesel sunumu `to be verified`.
- **3.1.3(c).** Kurumsal istisna, kuruluşlara çalışanları veya öğrencileri adına doğrudan satış koşuluna bağlıdır. Tüketici, tek kullanıcı ve aile satışları uygulama içi satın alma gerektirir. Hem Ev hem İşletme sunan GRISTORA'da yalnız web'den satılan İşletme katmanlarına bu istisnanın uygulanması `to be verified`. Onaylanana kadar İşletme ekranının önerilen kapsamı abonelik durumunu göstermektir. Mağazaya göre metin ve bağlantı izinleri `to be verified`.

Kaynak: [Apple App Review Guidelines, 3.1 Payments](https://developer.apple.com/app-store/review/guidelines/#payments), 26 Kasım 2026.

### 3. Plan anahtarı, koltuk sayısı ve modüller

Yetki kaydı çalışma alanının plan anahtarını (Ev veya İşletme ile katman) ve koltuk sayısını taşır. Sunucu doğrulanmış sağlayıcı ürün ve fiyat kimliklerini plan anahtarına eşler. Paddle İşletme aboneliğindeki adet, yetkideki koltuk sayısına dönüşür. İstemci modülleri, sunucunun plandan türettiği yetkiyle gösterir. Plan, koltuk ve kaynak değişiklikleri denetim kaydına yazılır.

Modüllerin mevcut P1 uygulaması için bkz. [#335](https://github.com/oguzbay-del/brewery/issues/335) ve [modül eşlemesi](../sunucu/crates/api/src/modules.rs). [ADR-0012](0012-urun-markasi.md) ürün markasını ele alır. Plan ile modül eşlemesinin karar kaydı için B2'de doğru başvuru belirlenir.

### 4. Ev'den İşletme'ye geçiş

1. Geçişi aynı çalışma alanında başlat. Mevcut ödeme kaynağını, etkin dönemi ve deneme bitişini göster.
2. App Store aboneliği varsa kullanıcı Apple üzerinden iptal eder. Otomatik yenilemenin kapatılması erişim süresini sona erdirmez. Paddle İşletme ödemesini Apple aboneliği artık aktif olmadığında başlat. Apple döneminin kalan kısmına ilişkin iade veya mahsup `to be verified`. Ev aboneliği Paddle'daysa İşletme katmanına Paddle plan değişikliğiyle geç.
3. Doğrulanmış Paddle değişikliği geldiğinde İşletme planı ve koltuk yetkisini aynı çalışma alanına bağla. Kaynak değişikliği yeni deneme başlatmaz. Bir aylık denemenin mevcut bitiş tarihi korunur.
4. Veriler, tarifler ve çalışma alanı kimliği korunur. Deneme ve abonelik hakkı yoksa ücretli özellikler salt okunur olur. Dışa aktarım devam eder.

## Sonuçlar

- Etkin abonelik kaynağının sahibi bellidir. İstemci ödeme veya erişim durumu için sunucuya bakar.
- Kaynak uyuşmazlığı desteğe gider. Geçişte kullanıcının mevcut dönemin bitmesini beklemesi gerekebilir.
- Plan ve koltuk eşlemeleri sağlayıcı ürün kataloglarıyla birlikte sürdürülür.

## Reddedilen seçenekler

- İki etkin ödeme kaynağı: Çift tahsilat ve çelişen yetki güncellemelerine yol açar.
- Koltuk ve modülü istemcide belirlemek: Web ve iOS farklı erişim kararı verebilir.
- Geçiş için yeni çalışma alanı: Mevcut kayıtları taşımayı ve bağlantıları yeniden kurmayı gerektirir.

## İzleyen işler

| İş | Kapsam |
| --- | --- |
| B2 | Yetki kaydına plan ve koltuk bilgisi, plan-modül eşlemesi ve denetim kaydı |
| B3 | Faturalama durumu, Paddle ödeme ve portal, App Store eşitleme uçları ve 409 |
| B4 | İki sağlayıcı olayından tek abonelik kaydı ve çift kaynak için destek kaydı |
| B5 | Web Abonelik ekranı, Paddle ödeme ve müşteri portalı |
| B6 | iOS Ev için StoreKit 2, geri yükleme ve yönetim, İşletme için durum |
| B7 | Deneme bitişi, ödeme başarısızlığı ve abonelik bitişi bildirimleri |

## Açık kararlar

- App Store incelemesi: Birleşik Ev ve İşletme uygulamasının 3.1.3(c) kapsamı, Ev için web ve iOS katman eşlemesi ve mağaza bazında İşletme yönetim metni veya bağlantısı.
- Oğuz: Kalan Apple dönemi için iade veya mahsup, bekleme süresinde İşletme erişiminin koşulları, plan katmanları, koltuk sınırları ve fiyatlar. Sağlayıcı ürün kimlikleri O10 ve O11 sırasında belirlenir.
